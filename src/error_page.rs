// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Error page module to compose an HTML page response.
//!

use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Method, Response, StatusCode, Uri};
use mime_guess::mime;
use std::path::Path;

use crate::{helpers, http_ext::MethodExt, Result};

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
                    String::from_utf8_lossy(&helpers::read_bytes_default(page404))
                        .trim()
                        .clone_into(&mut page_content);
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
                String::from_utf8_lossy(&helpers::read_bytes_default(page50x))
                    .trim()
                    .clone_into(&mut page_content);
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
        page_content = [
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
    let len = page_content.len() as u64;

    if !method.is_head() {
        body = Body::from(page_content)
    }

    let mut resp = Response::new(body);
    *resp.status_mut() = *status_code;
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    Ok(resp)
}
