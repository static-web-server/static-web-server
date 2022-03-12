use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt, HeaderValue};
use http::header::CONTENT_TYPE;
use hyper::{Body, Method, Response, StatusCode};

/// Checks if a fallback response can be generated, i.e. if it is a GET request that would result in a 404 error and a fallback page is configured.
/// If a response can be generated, it is returned, else `None` is returned.
pub fn fallback_response(
    method: &Method,
    status_code: &StatusCode,
    page_fallback: &Option<String>,
) -> Option<Response<Body>> {
    if (status_code == &StatusCode::NOT_FOUND) && (method == Method::GET) {
        if let Some(fallback_page_content) = page_fallback {
            let body = Body::from(fallback_page_content.to_owned());
            let len = fallback_page_content.len() as u64;

            let mut resp = Response::new(body);
            *resp.status_mut() = StatusCode::OK;
            resp.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            resp.headers_mut().typed_insert(ContentLength(len));
            resp.headers_mut().typed_insert(ContentType::html());
            resp.headers_mut().typed_insert(AcceptRanges::bytes());

            Some(resp)
        } else {
            None
        }
    } else {
        None
    }
}
