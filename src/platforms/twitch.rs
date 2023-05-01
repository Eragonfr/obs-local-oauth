use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, Scope,
    TokenUrl,
};

const SCOPES: &str = "channel:read:stream_key";
const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

pub fn get_redirect() {
    let mut client = BasicClient::new(
        ClientId::new("TWITCH_ID".to_string()),
        Some(ClientSecret::new("TWITCH_SECRET".to_string())),
        AuthUrl::new(TWITCH_AUTH_URL.to_string())?,
        Some(TokenUrl::new(TWITCH_TOKEN_URL.to_string())?), //client_secret: ctx.secret("TWITCH_SECRET")?.to_string(),
                                                            //redirect_url: ctx.var("TWITCH_REDIRECT_URL")?.to_string(),
                                                            //scope: SCOPES.to_string(),
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

    warp::redirect::temporary(auth_url)
}

pub async fn get_token(
    ctx: &RouteContext<()>,
    form_data: FormData,
    legacy: bool,
) -> Result<Response> {
    match get_twitch_config(ctx, legacy) {
        Ok(config) => oauth::get_token(config, form_data).await,
        Err(err) => Response::error(format!("Something went wrong: {}", err), 500),
    }
}
