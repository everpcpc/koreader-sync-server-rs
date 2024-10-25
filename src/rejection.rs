use warp::reject::Reject;

#[derive(Debug)]
pub(crate) enum Error {
    Unauthorized,
    Unknown,
}

impl Reject for Error {}
