use anyhow::Result;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code = if err.is_not_found() {
        StatusCode::NOT_FOUND
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        StatusCode::BAD_REQUEST
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        StatusCode::METHOD_NOT_ALLOWED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    let body = format!(
        "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
        code, code
    );
    Ok(warp::reply::with_status(warp::reply::html(body), code))
}
