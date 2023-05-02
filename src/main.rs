use std::{collections::HashMap, env::var};

//mod platforms;

//use platforms::restream;
//use platforms::twitch;

use lazy_static::lazy_static;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use warp::{http::StatusCode, redirect, Filter, Reply};

const BLANK_PAGE: &str = "This is an open field west of a white house, with a boarded front door.
There is a small mailbox here.
>";
const OAUTH_COMPLETE: &str = "OAuth process finished. This window should close momentarily.";

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

#[tokio::main]
async fn main() {
    // Prepare logging env
    elog::init_timed();

    let root = warp::any().map(|| warp::reply::html(BLANK_PAGE));
    let redirect = warp::path("v1")
        .and(warp::path("twitch"))
        .and(warp::path("redirect"))
        .map(get_redirect);
    let finalise =
        warp::path!("v1" / "twitch" / "finalise").map(|| warp::reply::html(OAUTH_COMPLETE));
    let token = warp::post()
        .and(warp::body::json())
        .and(warp::path!("v1" / "twitch" / "token"))
        .map(get_token);

    let routes = redirect.or(finalise).or(token).or(root);

    warp::serve(routes).run(([127, 0, 0, 1], 4433)).await;
}

fn get_redirect() -> impl Reply {
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

fn get_token(form_data: HashMap<String, String>) -> impl Reply {
    let mut post_data = vec![
        ("client_id", "TWITCH_ID".to_string()),
        ("client_secret", TWITCH_SECRET.to_string()),
        ("grant_type", form_data["grant_type"].clone()),
    ];

    let auth_url: String = "".to_string();
    let grant_type = "";

    match form_data["grant_type"].as_str() {
        "refresh_token" => {
            post_data.push(("refresh_token", form_data["refresh_token"].clone()));
        }
        "authorization_code" => {
            post_data.push(("code", form_data["code"].clone()));
            post_data.push(("redirect_uri", auth_url));
        }
        _ => {
            return warp::reply::html(format!("Invalid grant_type {}", grant_type)).into_response()
        }
    }

    // Make new request
    let client = reqwest::blocking::Client::new();
    let _resp = client.post(TWITCH_TOKEN_URL).json(&post_data).send();

    let resp = match _resp {
        Ok(r) => r,
        Err(e) => {
            let rep = warp::reply::json(&format!(
                "{{'error': 'internal_error', 'error_description': 'Fetch failed with {}'}}",
                e
            ));
            return warp::reply::with_status(rep, StatusCode::INTERNAL_SERVER_ERROR)
                .into_response();
        }
    };

    let status: u16 = resp.status().as_u16();
    let resp_json = resp.json::<HashMap<String, String>>();

    let data = match resp_json {
        Ok(j) => j,
        Err(e) => {
            let rep = warp::reply::json(&format!(
                "{{
                'error': 'parse_error',
                'error_description': Bad JSON response from {}: {}'
            }}",
                "TWITCH", &e
            ));
            return warp::reply::with_status(rep, StatusCode::INTERNAL_SERVER_ERROR)
                .into_response();
        }
    };

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
        return warp::reply::with_status(rep, StatusCode::from_u16(status).unwrap())
            .into_response();
    }

    let rep = warp::reply::json(&data);
    if data.contains_key("error") {
        warp::reply::with_status(rep, StatusCode::INTERNAL_SERVER_ERROR).into_response()
    } else {
        warp::reply::with_status(rep, StatusCode::OK).into_response()
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
