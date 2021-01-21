use anyhow::Result;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

// It receives a `Rejection` and tries to return a HTML error reply.
pub async fn handle_rejection(
    page_404: String,
    page_50x: String,
    err: Rejection,
) -> Result<impl Reply, Infallible> {
    let mut content = String::new();
    let code = if err.is_not_found() {
        content = page_404;
        StatusCode::NOT_FOUND
    } else {
        if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
        {
            StatusCode::BAD_REQUEST
        } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
            StatusCode::METHOD_NOT_ALLOWED
        } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        } else {
            content = page_50x;
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };

    if content.is_empty() {
        content = format!(
            "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
            code, code
        );
    }

    Ok(warp::reply::with_status(warp::reply::html(content), code))
}
