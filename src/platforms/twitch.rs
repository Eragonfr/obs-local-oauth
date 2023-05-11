use std::{collections::HashMap, env::var};

use lazy_static::lazy_static;
use log::{debug, trace};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use serde::{Deserialize, Serialize};
use warp::{http::StatusCode, redirect, Rejection, Reply};

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

lazy_static! {
    static ref TWITCH_CLIENT_ID: String = match var("TWITCH_CLIENT_ID") {
        Ok(t) => t,
        Err(_) => panic!("TWITCH_CLIENT_ID not found in current env."),
    };
    static ref TWITCH_SECRET: String = match var("TWITCH_SECRET") {
        Ok(t) => t,
        Err(_) => panic!("TWITCH_SECRET variable not found in current env."),
    };
    static ref TWITCH_REDIRECT_URL: String = match var("TWITCH_REDIRECT_URL") {
        Ok(t) => t,
        Err(_) => panic!("TWITCH_REDIRECT_URL not found in env"),
    };
}

#[derive(Debug, Serialize)]
struct TokenRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    refresh_token: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TwitchTokenResponse {
    status: Option<f64>,
    message: Option<String>,
    error: Option<String>,
    access_token: Option<String>,
    expire_in: Option<f64>,
    refresh_token: Option<String>,
    scope: Option<Vec<String>>,
    token_type: Option<String>,
}

pub fn get_redirect() -> impl Reply {
    debug!("Redirect endpoint requested");
    let client = BasicClient::new(
        ClientId::new(TWITCH_CLIENT_ID.to_string()),
        Some(ClientSecret::new(TWITCH_SECRET.to_string())),
        AuthUrl::new(TWITCH_AUTH_URL.to_string()).unwrap(),
        Some(TokenUrl::new(TWITCH_TOKEN_URL.to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(TWITCH_REDIRECT_URL.to_string()).unwrap());

    let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("channel:read:stream_key".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    let auth_url_string: String = auth_url.to_string();

    redirect::temporary(warp::http::Uri::from_maybe_shared(auth_url_string).unwrap())
        .into_response()
}

pub async fn get_token(form_data: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    debug!("Token endpoint requested");
    let grant_type = &form_data["grant_type"];
    let mut post_data = TokenRequest {
        client_id: TWITCH_CLIENT_ID.to_string(),
        client_secret: TWITCH_SECRET.to_string(),
        grant_type: grant_type.to_string(),
        refresh_token: None,
        code: None,
        redirect_uri: None,
    };

    let auth_url: String = "http://localhost:4433/v1/twitch/finalise/".to_string();

    match grant_type.as_str() {
        "refresh_token" => {
            post_data.refresh_token = Some(form_data["refresh_token"].to_string());
        }
        "authorization_code" => {
            post_data.code = Some(form_data["code"].to_string());
            post_data.redirect_uri = Some(auth_url);
        }
        _ => {
            return Ok(
                warp::reply::html(format!("Invalid grant_type {}", grant_type)).into_response(),
            )
        }
    }

    trace!("{:?}", post_data);

    let client = reqwest::Client::new();
    let _resp = client.post(TWITCH_TOKEN_URL).json(&post_data).send().await;

    let resp = match _resp {
        Ok(r) => r,
        Err(e) => {
            let rep = warp::reply::json(&format!(
                "{{'error': 'internal_error', 'error_description': 'Fetch failed with {}'}}",
                e
            ));
            return Ok(
                warp::reply::with_status(rep, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            );
        }
    };

    let status: u16 = resp.status().as_u16();
    let resp_json = resp.json::<TwitchTokenResponse>().await;
    trace!("{:?}", resp_json);

    let data = match resp_json {
        Ok(j) => j,
        Err(e) => {
            let rep = warp::reply::json(&format!(
                "{{ 'error': 'parse_error', 'error_description': Bad JSON response from {}: {}' }}",
                "TWITCH", &e
            ));
            return Ok(
                warp::reply::with_status(rep, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            );
        }
    };

    if status != 200 {
        let resp_data: String;

        if data.message.is_some() {
            let message = data
                .message
                .expect("WTF Rust, you said the message field was populated");
            if message.as_str() == "Invalid refresh token" {
                resp_data = format!("{{ 'error': 'Error', 'error_description': 'Your {} login token is no longer valid. Please try reconnecting your account.' }}", "TWITCH");
            } else {
                resp_data = format!("{{ 'error': 'Error', 'error_description': {} }}", message)
            };
        } else {
            resp_data = format!(
                "{{ 'error': 'status_error', 'error_description': Received HTTP {} from {} }}",
                status, "TWITCH"
            );
        }
        let rep = warp::reply::json(&resp_data);
        return Ok(
            warp::reply::with_status(rep, StatusCode::from_u16(status).unwrap()).into_response(),
        );
    }

    let rep = warp::reply::json(&data);
    if data.error.is_some() {
        Ok(warp::reply::with_status(rep, StatusCode::INTERNAL_SERVER_ERROR).into_response())
    } else {
        Ok(warp::reply::with_status(rep, StatusCode::OK).into_response())
    }
}
