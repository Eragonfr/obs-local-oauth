use std::{collections::HashMap, env::var};

use lazy_static::lazy_static;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, Scope,
    TokenUrl,
};
use warp::{Reply, http::StatusCode, redirect};

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

lazy_static! {
    static ref TWITCH_SECRET: String = match var("TWITCH_SECRET") {
        Ok(t) => t,
        Err(_) => panic!("TWITCH_SECRET variable not found in current env."),
    };
    static ref TWITCH_REDIRECT_URL: String = match var("TWITCH_REDIRECT_URL") {
        Ok(t) => t,
        Err(_) => panic!("TWITCH_REDIRECT_URL not found in env"),
    };
}

pub fn get_redirect() -> impl Reply {
    let mut client = BasicClient::new(
        ClientId::new("TWITCH_ID".to_string()),
        Some(ClientSecret::new(TWITCH_SECRET.to_string())),
        AuthUrl::new(TWITCH_AUTH_URL.to_string()).unwrap(),
        Some(TokenUrl::new(TWITCH_TOKEN_URL.to_string()).unwrap()),
    );

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("chanel".to_string()))
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("stream_key".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    return Ok(redirect::temporary(auth_url).into_response());
}

pub fn get_token(form_data: HashMap<String, String>) -> impl Reply {
    let mut post_data = vec![
        ("client_id", "TWITCH_ID".to_string()),
        ("client_secret", TWITCH_SECRET.to_string()),
        ("grant_type", form_data["grant_type"]),
    ];

    let auth_url: String = "".to_string();
    let grant_type = "";

    match form_data["grant_type"].as_str() {
        "refresh_token" => {
            post_data.push(("refresh_token", form_data["refresh_token"]));
        }
        "authorization_code" => {
            post_data.push(("code", form_data["code"]));
            post_data.push(("redirect_uri", auth_url));
        }
        _ => return Err(format!("Invalid grant_type {}", grant_type)),
    }

    // Make new request
    let client = reqwest::blocking::Client::new();
    let _resp = client.post(TWITCH_TOKEN_URL).json(&post_data).send();

    let mut resp = match _resp {
        Ok(r) => r,
        Err(e) => {
            let rep = warp::reply::json(&format!(
                "{{'error': 'internal_error', 'error_description': 'Fetch failed with {}'}}",
                e
            ));
            return Ok(warp::reply::with_status(
                rep,
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let data = match resp.json::<HashMap<String, String>>() {
        Ok(j) => j,
        Err(e) => {
            let rep = warp::reply::json(&format!(
                "{{
                'error': 'parse_error',
                'error_description': Bad JSON response from {}: {}'
            }}",
                "TWITCH", &e
            ));
            return Ok(warp::reply::with_status(
                rep,
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let status = resp.status();
    if status != 200 {
        let resp_data: String;

        if data.contains_key("message") {
            if data["message"].as_str() == "Invalid refresh token" {
                resp_data = format!("{{
                    'error': 'Error',
                    'error_description': 'Your {} login token is no longer valid. Please try reconnecting your account.'
                }}", "TWITCH_ID");
            } else {
                resp_data = format!(
                    "{{
                    'error': 'Error',
                    'error_description': {}
                }}",
                    data["message"]
                )
            };
        } else {
            resp_data = format!(
                "{{
                'error': 'status_error',
                'error_description': Received HTTP {} from {}
            }}",
                status, "TWITCH_ID"
            );
        }
        let rep = warp::reply::json(&resp_data);
        return Ok(warp::reply::with_status(rep, status));
    }

    let rep = warp::reply::json(&data);
    if data.contains_key("error") {
        return Ok(warp::reply::with_status(
            rep,
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    } else {
        return Ok(warp::reply::with_status(rep, StatusCode::OK));
    }

    /*
    if let Err(err) = res {
        // Assume that if we're here the request was missing a required parameter
        let res = Response::from_json({
            "error": "request_error",
            "error_description": format!("Request failed due to the following error: {}", err),
        });
        return res.with_status(400);
    }*/
}
