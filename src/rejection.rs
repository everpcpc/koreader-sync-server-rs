use redis::RedisError;
use warp::http::StatusCode;
use warp::reject::Reject;

#[derive(Debug)]
pub(crate) enum Error {
    InvalidField(String),
    UserExists(String),
    Redis(RedisError),
    Unauthorized,
    Unknown(String),
}

impl Reject for Error {}

impl Error {
    pub fn response(&self) -> (String, StatusCode) {
        match self {
            Error::Unauthorized => ("UNAUTHORIZED".to_string(), StatusCode::UNAUTHORIZED),
            Error::Redis(err) => {
                log::error!("redis error: {:?}", err);
                (
                    "INTERNAL_SERVER_ERROR".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
            Error::InvalidField(field) => {
                (format!("INVALID_FIELD: {}", field), StatusCode::BAD_REQUEST)
            }
            Error::UserExists(user) => (format!("USER_EXISTS: {}", user), StatusCode::CONFLICT),
            Error::Unknown(err) => {
                log::error!("unknown error: {:?}", err);
                (
                    "INTERNAL_SERVER_ERROR".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    }
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Self {
        Error::Redis(err)
    }
}
