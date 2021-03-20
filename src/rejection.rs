use anyhow::Result;
use once_cell::sync::OnceCell;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

pub static PAGE_404: OnceCell<String> = OnceCell::new();
pub static PAGE_50X: OnceCell<String> = OnceCell::new();

/// It receives a `Rejection` and tries to return the corresponding HTML error reply.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let mut content = String::new();
    let code = if err.is_not_found() {
        content = PAGE_404
            .get()
            .expect("page 404 is not initialized")
            .to_string();
        StatusCode::NOT_FOUND
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        StatusCode::BAD_REQUEST
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        StatusCode::METHOD_NOT_ALLOWED
    } else if err.find::<warp::filters::cors::CorsForbidden>().is_some() {
        StatusCode::FORBIDDEN
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    } else {
        content = PAGE_50X
            .get()
            .expect("page 50x is not initialized")
            .to_string();
        StatusCode::INTERNAL_SERVER_ERROR
    };

    if content.is_empty() {
        content = format!(
            "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
            code, code
        );
    }

    Ok(warp::reply::with_status(warp::reply::html(content), code))
}
