use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Method, Response, StatusCode};
use once_cell::sync::OnceCell;

use crate::error::Result;

pub static PAGE_404: OnceCell<String> = OnceCell::new();
pub static PAGE_50X: OnceCell<String> = OnceCell::new();

pub fn get_error_response(method: &Method, status_code: &StatusCode) -> Result<Response<Body>> {
    tracing::warn!(method = ?method, status = status_code.as_u16(), error = ?status_code.to_string());

    // Check for 4xx and 50x status codes
    let mut error_page_content = String::new();
    let status_code = match status_code {
        // 4xx
        &StatusCode::BAD_REQUEST
        | &StatusCode::UNAUTHORIZED
        | &StatusCode::PAYMENT_REQUIRED
        | &StatusCode::FORBIDDEN
        | &StatusCode::NOT_FOUND
        | &StatusCode::METHOD_NOT_ALLOWED
        | &StatusCode::NOT_ACCEPTABLE
        | &StatusCode::PROXY_AUTHENTICATION_REQUIRED
        | &StatusCode::REQUEST_TIMEOUT
        | &StatusCode::CONFLICT
        | &StatusCode::GONE
        | &StatusCode::LENGTH_REQUIRED
        | &StatusCode::PRECONDITION_FAILED
        | &StatusCode::PAYLOAD_TOO_LARGE
        | &StatusCode::URI_TOO_LONG
        | &StatusCode::UNSUPPORTED_MEDIA_TYPE
        | &StatusCode::RANGE_NOT_SATISFIABLE
        | &StatusCode::EXPECTATION_FAILED => {
            // Extra check for 404 status code and HTML content
            if status_code == &StatusCode::NOT_FOUND {
                // TODO: handle this potential panic properly
                error_page_content = PAGE_404
                    .get()
                    .expect("PAGE_404 contant is not initialized")
                    .to_string();
            }
            status_code
        }
        // 50x
        &StatusCode::INTERNAL_SERVER_ERROR
        | &StatusCode::NOT_IMPLEMENTED
        | &StatusCode::BAD_GATEWAY
        | &StatusCode::SERVICE_UNAVAILABLE
        | &StatusCode::GATEWAY_TIMEOUT
        | &StatusCode::HTTP_VERSION_NOT_SUPPORTED
        | &StatusCode::VARIANT_ALSO_NEGOTIATES
        | &StatusCode::INSUFFICIENT_STORAGE
        | &StatusCode::LOOP_DETECTED => {
            // HTML content for error status codes 50x
            // TODO: handle this potential panic properly
            error_page_content = PAGE_50X
                .get()
                .expect("PAGE_50X contant is not initialized")
                .to_string();
            status_code
        }
        // other status codes
        _ => status_code,
    };

    if error_page_content.is_empty() {
        error_page_content = format!(
            "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
            status_code, status_code
        );
    }

    let mut body = Body::empty();
    let len = error_page_content.len() as u64;

    if method != Method::HEAD {
        body = Body::from(error_page_content)
    }

    let mut resp = Response::new(body);
    *resp.status_mut() = *status_code;
    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(ContentType::html());
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    Ok(resp)
}
