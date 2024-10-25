use std::convert::Infallible;

use argh::FromArgs;
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::Filter;

mod dto;
mod filters;
mod handlers;
mod rejection;
mod utils;

use crate::rejection::Error;

fn default_redis() -> String {
    "redis://127.0.0.1:6379/0".to_string()
}

#[derive(FromArgs, Debug)]
/// synchronization service for koreader devices
struct Config {
    /// port to listen on, defaults: 3030
    #[argh(option, short = 'p', default = "3030")]
    port: u16,

    /// redis server address for storage, defaults: redis://127.0.0.1:6379/0
    #[argh(option, short = 'r', default = "default_redis()")]
    redis: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let cfg: Config = argh::from_env();

    let db = redis::Client::open(cfg.redis).expect("could not create redis client");

    let api = filters::routes(db)
        .recover(|err: Rejection| async move {
            let (reply, code) = if let Some(err) = err.find::<Error>() {
                err.response()
            } else {
                log::error!("unhandled rejection: {:?}", err);
                (
                    "INTERNAL_SERVER_ERROR".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            };
            Ok::<_, Infallible>(warp::reply::with_status(reply, code))
        })
        .with(warp::log("kosync"));
    warp::serve(api).run(([0, 0, 0, 0], cfg.port)).await;
}
