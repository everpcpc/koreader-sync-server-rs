use std::collections::HashMap;
use std::convert::Infallible;

use redis::AsyncCommands;
use redis::Client;
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::Reply;

use crate::dto;
use crate::rejection::Error;
use crate::utils::is_valid_field;
use crate::utils::is_valid_key_field;

pub async fn get_conn(client: Client) -> Result<impl AsyncCommands, Rejection> {
    let conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(Error::from)?;
    Ok(conn)
}

fn user_key(user: &str) -> String {
    format!("user:{}:key", user)
}

fn doc_key(user: &str, doc: &str) -> String {
    format!("user:{}:document:{}", user, doc)
}

pub async fn authorize(
    mut rds: impl AsyncCommands,
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
    if !is_valid_key_field(&user) || !is_valid_field(&key) {
        return Err(warp::reject::custom(Error::Unauthorized));
    }
    let auth_key: String = rds
        .get(user_key(&user))
        .await
        .map_err(|_| warp::reject::custom(Error::Unauthorized))?;
    if auth_key != key {
        return Err(warp::reject::custom(Error::Unauthorized));
    }
    Ok(user)
}

pub async fn healthcheck() -> Result<impl Reply, Infallible> {
    Ok("ok")
}

pub async fn auth_user(_: String) -> Result<impl Reply, Infallible> {
    let resp = HashMap::from([("authorized", "OK")]);
    Ok(warp::reply::json(&resp))
}

pub async fn create_user(
    mut rds: impl AsyncCommands,
    req: dto::CreateUserRequest,
) -> Result<impl Reply, Rejection> {
    req.validate()?;
    let uk = user_key(&req.username);
    let exists: bool = rds.exists(&uk).await.map_err(Error::from)?;
    if exists {
        return Err(warp::reject::custom(Error::UserExists(
            req.username.clone(),
        )));
    }
    let ok: bool = rds.set(&uk, &req.password).await.map_err(Error::from)?;
    if !ok {
        return Err(warp::reject::custom(Error::Unknown(
            "could not create user".to_string(),
        )));
    }
    let resp = HashMap::from([("username", req.username)]);
    Ok(warp::reply::with_status(
        warp::reply::json(&resp),
        StatusCode::CREATED,
    ))
}

pub async fn get_progress(
    doc: String,
    username: String,
    mut rds: impl AsyncCommands,
) -> Result<impl warp::Reply, Rejection> {
    if !is_valid_key_field(&doc) {
        return Err(warp::reject::custom(Error::InvalidField(
            "document".to_string(),
        )));
    }
    let dk = doc_key(&username, &doc);
    let resp: HashMap<String, String> = rds.hgetall(&dk).await.map_err(Error::from)?;
    let progress = dto::Progress::from(resp);
    Ok(warp::reply::json(&progress))
}

pub async fn update_progress(
    username: String,
    mut rds: impl AsyncCommands,
    mut progress: dto::Progress,
) -> Result<impl Reply, Rejection> {
    progress.validate()?;
    progress.timestamp = Some(chrono::Utc::now().timestamp() as u64);
    let dk = doc_key(&username, &progress.document);
    let ok: bool = rds
        .hset_multiple(&dk, &progress.to_vec())
        .await
        .map_err(Error::from)?;
    if !ok {
        return Err(warp::reject::custom(Error::Unknown(
            "could not update progress".to_string(),
        )));
    }
    Ok(warp::reply::json(&progress))
}
