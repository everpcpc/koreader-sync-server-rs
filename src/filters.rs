use redis::{aio::MultiplexedConnection, Client};
use warp::{Filter, Rejection, Reply};

use crate::handlers;

pub fn routes(client: Client) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    healthcheck()
        .or(create_user(client.clone()))
        .or(auth_user(client.clone()))
        .or(update_progress(client.clone()))
        .or(get_progress(client))
}

fn with_redis(
    client: Client,
) -> impl Filter<Extract = (MultiplexedConnection,), Error = Rejection> + Clone {
    warp::any()
        .map(move || client.clone())
        .and_then(handlers::get_conn)
}

fn with_auth(client: Client) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::any()
        .and(with_redis(client))
        .and(warp::header::optional("x-auth-user"))
        .and(warp::header::optional("x-auth-key"))
        .and_then(handlers::authorize)
}

pub fn healthcheck() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("healthcheck")
        .and(warp::get())
        .and_then(handlers::healthcheck)
}

pub fn create_user(
    client: Client,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("users" / "create")
        .and(warp::post())
        .and(with_redis(client))
        .and(warp::body::json())
        .and_then(handlers::create_user)
}

pub fn auth_user(
    client: Client,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("users" / "auth")
        .and(warp::get())
        .and(with_auth(client))
        .and_then(handlers::auth_user)
}

pub fn update_progress(
    client: Client,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("syncs" / "progress")
        .and(warp::post())
        .and(with_auth(client.clone()))
        .and(with_redis(client))
        .and(warp::body::json())
        .and_then(handlers::update_progress)
}

pub fn get_progress(
    client: Client,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("syncs" / "progress" / String)
        .and(warp::get())
        .and(with_auth(client.clone()))
        .and(with_redis(client))
        .and_then(handlers::get_progress)
}
