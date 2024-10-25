use std::convert::Infallible;

use argh::FromArgs;
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::reply;
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

fn get_rejection_status(err: &Rejection) -> StatusCode {
    if err.is_not_found() {
        StatusCode::NOT_FOUND
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        StatusCode::METHOD_NOT_ALLOWED
    } else if err.find::<warp::reject::InvalidHeader>().is_some()
        || err.find::<warp::reject::MissingHeader>().is_some()
        || err.find::<warp::reject::MissingCookie>().is_some()
        || err.find::<warp::reject::InvalidQuery>().is_some()
        || err.find::<warp::body::BodyDeserializeError>().is_some()
        || err.find::<warp::ws::MissingConnectionUpgrade>().is_some()
    {
        StatusCode::BAD_REQUEST
    } else if err.find::<warp::reject::LengthRequired>().is_some() {
        StatusCode::LENGTH_REQUIRED
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        StatusCode::PAYLOAD_TOO_LARGE
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    } else if err.find::<warp::cors::CorsForbidden>().is_some() {
        StatusCode::FORBIDDEN
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let cfg: Config = argh::from_env();

    let db = redis::Client::open(cfg.redis).expect("could not create redis client");

    let api = filters::routes(db)
        .recover(|err: Rejection| async move {
            if let Some(e) = err.find::<Error>() {
                let (msg, status) = e.response();
                Ok::<_, Infallible>(reply::with_status(msg, status))
            } else {
                let status = get_rejection_status(&err);
                if status == StatusCode::INTERNAL_SERVER_ERROR {
                    log::error!("unhandled rejection: {:?}", err);
                }
                Ok::<_, Infallible>(reply::with_status(String::new(), status))
            }
        })
        .with(warp::log("kosync"));
    warp::serve(api).run(([0, 0, 0, 0], cfg.port)).await;
}
