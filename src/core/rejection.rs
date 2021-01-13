use anyhow::Result;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

// It receives a `Rejection` and tries to return a HTML error reply.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code = if err.is_not_found() {
        StatusCode::NOT_FOUND
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        StatusCode::BAD_REQUEST
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        StatusCode::METHOD_NOT_ALLOWED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    let content = format!(
        "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
        code, code
    );
    Ok(warp::reply::with_status(warp::reply::html(content), code))
}
