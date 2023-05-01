use reqwest::{Request, Response};
use warp::Filter;

mod platforms;

//use platforms::restream;
use platforms::twitch;

const BLANK_PAGE: &str = "This is an open field west of a white house, with a boarded front door.
There is a small mailbox here.
>";
const FOUR_OH_FOUR: &str = "This is an open field.
There is nothing here.
>";
const OAUTH_COMPLETE: &str = "OAuth process finished. This window should close momentarily.";

#[tokio::main]
async fn main() {
    let root = warp::any().and_then(/* BLANK_PAGE*/);
    let v1 = warp::path!("v1");
    let redirect = v1
        .and(warp::path!(String / "redirect"))
        .and_then(handle_redirects);
    let finalise = v1.and(warp::path!(String / "finalise")).and_then(/* OAUTH_COMPLETE */);
    let token = v1.and(warp::path!(String / "token")).and_then(handle_token);

    let routes = redirect.or(finalise).or(token).or(root);

    warp::serve(routes).run([127, 0, 0, 1], 4433).await;
}

fn handle_redirects(platform: String) -> Result<impl warp::Reply, warp::Error> {
    match platform.as_str() {
        "twitch" => Ok(twitch::get_redirect(&"", false)),
        //        "restream" => restream::get_redirect(&ctx, false),
        _ => Err(format!("Unknown platform: {}", platform)),
    }
}

async fn handle_token(platform: String) -> Result<impl warp::Reply, warp::Error> {
    let form_data = Err("TODO"); // req.form_data().await;
    if let Err(err) = form_data {
        return Err(format!("Bad Request: {}", err));
    }

    match platform.as_str() {
        "twitch" => Ok(twitch::get_token(&"", form_data?, false)),
        //        "restream" => restream::get_token(&ctx, form_data?, false).await,
        _ => Err(format!("Unknown platform: {}", platform)),
    }
}
