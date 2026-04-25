// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Error page module to compose an HTML page response.
//!

use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Method, Response, StatusCode, Uri};
use maud::{DOCTYPE, html};
use mime_guess::mime;
use std::path::Path;

use crate::body::Body;
use crate::{Result, helpers, http_ext::MethodExt};

/// Build an `text/html` response with the correct `Content-Length` and
/// `Accept-Ranges` headers.
///
/// When `method` is `HEAD` the body is omitted but the `Content-Length`
/// still reflects the full content length, exactly as for a normal `GET`.
/// Pass `method = None` to always include the body (e.g. fallback pages
/// where the caller never receives `HEAD` requests at this point).
pub(crate) fn build_html_response(
    content: impl Into<bytes::Bytes>,
    status: hyper::StatusCode,
    method: Option<&Method>,
) -> Response<Body> {
    let bytes: bytes::Bytes = content.into();
    let len = bytes.len() as u64;
    let is_head = method.is_some_and(|m| m.is_head());
    let body = if is_head {
        crate::body::empty()
    } else {
        crate::body::full(bytes)
    };
    let mut resp = Response::new(body);
    *resp.status_mut() = status;
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());
    resp
}

/// It returns a HTTP error response which also handles available `404` or `50x` HTML content.
pub fn error_response(
    uri: &Uri,
    method: &Method,
    status_code: &StatusCode,
    page404: &Path,
    page50x: &Path,
) -> Result<Response<Body>> {
    tracing::warn!(
        method = ?method, uri = ?uri, status = status_code.as_u16(),
        error = status_code.canonical_reason().unwrap_or_default()
    );

    // Check for 4xx/50x status codes and handle their corresponding HTML content
    let mut page_content = String::new();
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
            if status_code == &StatusCode::NOT_FOUND {
                if page404.is_file() {
                    helpers::read_text_default(page404).clone_into(&mut page_content);
                } else {
                    tracing::debug!(
                        "page404 file path not found or not a regular file: {}",
                        page404.display()
                    );
                }
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
            if page50x.is_file() {
                helpers::read_text_default(page50x).clone_into(&mut page_content);
            } else {
                tracing::debug!(
                    "page50x file path not found or not a regular file: {}",
                    page50x.display()
                );
            }
            status_code
        }
        // other status codes
        _ => status_code,
    };

    if page_content.is_empty() {
        let reason = status_code.canonical_reason().unwrap_or_default();
        let title = [status_code.as_str(), " ", reason].concat();

        page_content = html! {
            (DOCTYPE)
            html {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content="width=device-width,minimum-scale=1,initial-scale=1";
                    title {
                        (title)
                    }
                    style {
                        "html { color-scheme: light dark; } body { font-family: sans-serif; text-align: center; }"
                    }
                }
                body {
                    h1 {
                        (title)
                    }
                }
            }
        }.into();
    }

    Ok(build_html_response(
        page_content,
        *status_code,
        Some(method),
    ))
}

#[cfg(test)]
mod tests {
    use headers::{ContentLength, ContentType, HeaderMapExt};
    use hyper::{Method, StatusCode};
    use std::path::Path;

    use super::{build_html_response, error_response};

    #[test]
    fn build_html_response_get_includes_body() {
        let resp = build_html_response("hello", StatusCode::OK, Some(&Method::GET));
        assert_eq!(resp.status(), StatusCode::OK);
        let ct: ContentType = resp.headers().typed_get().unwrap();
        assert_eq!(ct, ContentType::from(mime_guess::mime::TEXT_HTML_UTF_8));
        let cl: ContentLength = resp.headers().typed_get().unwrap();
        assert_eq!(cl.0, 5);
    }

    #[test]
    fn build_html_response_head_omits_body_but_keeps_length() {
        let content = "hello";
        let resp = build_html_response(content, StatusCode::NOT_FOUND, Some(&Method::HEAD));
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let cl: ContentLength = resp.headers().typed_get().unwrap();
        assert_eq!(cl.0, content.len() as u64);
    }

    #[test]
    fn build_html_response_none_method_always_includes_body() {
        let resp = build_html_response("body content", StatusCode::INTERNAL_SERVER_ERROR, None);
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let cl: ContentLength = resp.headers().typed_get().unwrap();
        assert_eq!(cl.0, "body content".len() as u64);
    }

    #[test]
    fn error_response_404_no_custom_page() {
        let uri = "/missing".parse().unwrap();
        let resp = error_response(
            &uri,
            &Method::GET,
            &StatusCode::NOT_FOUND,
            Path::new("/nonexistent/404.html"),
            Path::new("/nonexistent/50x.html"),
        )
        .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_response_404_with_custom_page() {
        let page404 = std::env::temp_dir().join("sws_error_page_404_test.html");
        std::fs::write(&page404, b"<h1>Not Found</h1>").unwrap();
        let uri = "/missing".parse().unwrap();
        let resp = error_response(
            &uri,
            &Method::GET,
            &StatusCode::NOT_FOUND,
            &page404,
            Path::new("/nonexistent/50x.html"),
        )
        .unwrap();
        std::fs::remove_file(&page404).ok();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_response_500_no_custom_page() {
        let uri = "/crash".parse().unwrap();
        let resp = error_response(
            &uri,
            &Method::GET,
            &StatusCode::INTERNAL_SERVER_ERROR,
            Path::new("/nonexistent/404.html"),
            Path::new("/nonexistent/50x.html"),
        )
        .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_response_head_omits_body() {
        let uri = "/missing".parse().unwrap();
        let resp = error_response(
            &uri,
            &Method::HEAD,
            &StatusCode::NOT_FOUND,
            Path::new("/nonexistent/404.html"),
            Path::new("/nonexistent/50x.html"),
        )
        .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let cl: ContentLength = resp.headers().typed_get().unwrap();
        assert!(
            cl.0 > 0,
            "Content-Length should reflect body size even for HEAD"
        );
    }
}
