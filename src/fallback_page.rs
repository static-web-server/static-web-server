use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Response, StatusCode};
use mime_guess::mime;
/// Checks if a fallback response can be generated, i.e. if it is a GET request that would result in a 404 error and a fallback page is configured.
/// If a response can be generated, it is returned, else `None` is returned.
pub fn fallback_response(page_fallback: &str) -> Response<Body> {
    let body = Body::from(page_fallback.to_owned());
    let len = page_fallback.len() as u64;

    let mut resp = Response::new(body);
    *resp.status_mut() = StatusCode::OK;

    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    resp
}
