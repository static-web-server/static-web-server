use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;

use crate::Result;

/// It returns a HTTP error response which also handles available `404` or `50x` HTML content.
pub fn error_response(
    method: &Method,
    status_code: &StatusCode,
    page404: &[u8],
    page50x: &[u8],
) -> Result<Response<Body>> {
    tracing::warn!(method = ?method, status = status_code.as_u16(), error = ?status_code.to_owned());

    // Check for 4xx/50x status codes and handle their corresponding HTML content
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
            // Extra check for 404 status code and its HTML content
            if status_code == &StatusCode::NOT_FOUND && !page404.is_empty() {
                error_page_content = String::from_utf8_lossy(page404).to_string();
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
            // HTML content check for status codes 50x
            if !page50x.is_empty() {
                error_page_content = String::from_utf8_lossy(page50x).to_string();
            }
            status_code
        }
        // other status codes
        _ => status_code,
    };

    if error_page_content.is_empty() {
        error_page_content = [
            "<html><head><title>",
            status_code.as_str(),
            " ",
            status_code.canonical_reason().unwrap_or_default(),
            "</title></head><body><center><h1>",
            status_code.as_str(),
            " ",
            status_code.canonical_reason().unwrap_or_default(),
            "</h1></center></body></html>",
        ]
        .concat();
    }

    let mut body = Body::empty();
    let len = error_page_content.len() as u64;

    if method != Method::HEAD {
        body = Body::from(error_page_content)
    }

    let mut resp = Response::new(body);
    *resp.status_mut() = *status_code;
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    Ok(resp)
}
