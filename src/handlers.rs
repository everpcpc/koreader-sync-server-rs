use std::convert::Infallible;

use redis::{aio::MultiplexedConnection, Client};
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::Reply;

use crate::rejection::Error;

static USER_KEY: &str = "user:{}:key";
static DOC_KEY: &str = "user:{}:document:{}";

pub async fn get_conn(client: Client) -> Result<MultiplexedConnection, Rejection> {
    let conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| {
            log::error!("could not get redis connection: {}", err);
            warp::reject::custom(Error::Unknown)
        })?;
    Ok(conn)
}

fn is_valid_key_field(field: &str) -> bool {
    !field.is_empty() && !field.contains(":")
}

pub async fn authorize(
    rds: MultiplexedConnection,
    user: Option<String>,
    key: Option<String>,
) -> Result<String, Rejection> {
    let user = match user {
        Some(user) => user,
        None => return Err(warp::reject::custom(Error::Unauthorized)),
    };
    let key = match key {
        Some(key) => key,
        None => return Err(warp::reject::custom(Error::Unauthorized)),
    };
    if user.is_empty() || key.is_empty() {
        return Err(warp::reject::custom(Error::Unauthorized));
    }
    if !is_valid_key_field(&user) || key.is_empty() {
        return Err(warp::reject::custom(Error::Unauthorized));
    }
    Ok(user)
}

pub async fn healthcheck() -> Result<impl Reply, Infallible> {
    Ok("ok")
}

pub async fn create_user(rds: MultiplexedConnection, _: ()) -> Result<impl Reply, Infallible> {
    todo!();
    Ok(StatusCode::CREATED)
}

pub async fn auth_user(username: String) -> Result<impl Reply, Infallible> {
    Ok(username)
}

pub async fn update_progress(
    username: String,
    rds: MultiplexedConnection,
    _: (),
) -> Result<impl Reply, Infallible> {
    todo!();
    Ok(StatusCode::OK)
}

pub async fn get_progress(
    _: String,
    username: String,
    rds: MultiplexedConnection,
) -> Result<impl warp::Reply, Infallible> {
    todo!();
    Ok(StatusCode::OK)
}
