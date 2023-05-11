mod platforms;

use platforms::twitch;

use warp::Filter;

const BLANK_PAGE: &str = "This is an open field west of a white house, with a boarded front door.
There is a small mailbox here.
>";
const OAUTH_COMPLETE: &str = "OAuth process finished. This window should close momentarily.";

#[tokio::main]
async fn main() {
    // Prepare logging env
    elog::init_timed();

    let root = warp::any().map(|| warp::reply::html(BLANK_PAGE));
    let redirect = warp::path!("v1" / "twitch" / "redirect").map(twitch::get_redirect);
    let finalise =
        warp::path!("v1" / "twitch" / "finalise").map(|| warp::reply::html(OAUTH_COMPLETE));
    let token = warp::post()
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::form())
        .and(warp::path!("v1" / "twitch" / "token"))
        .and_then(twitch::get_token);

    let routes = redirect.or(finalise).or(token).or(root);

    warp::serve(routes).run(([127, 0, 0, 1], 4433)).await;
}
