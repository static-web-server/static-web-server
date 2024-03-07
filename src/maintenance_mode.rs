// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Provides maintenance mode functionality.
//!

use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;
use std::path::Path;

use crate::{helpers, http_ext::MethodExt, Result};

const DEFAULT_BODY_CONTENT: &str = "The server is under maintenance mode";

/// Get the a server maintenance mode response.
pub fn get_response(
    method: &Method,
    status_code: &StatusCode,
    file_path: &Path,
) -> Result<Response<Body>> {
    tracing::debug!("server has entered into maintenance mode");
    tracing::debug!("maintenance mode file path to use: {}", file_path.display());

    let mut body_content = String::new();
    if file_path.is_file() {
        body_content = String::from_utf8_lossy(&helpers::read_bytes_default(file_path))
            .to_string()
            .trim()
            .to_owned();
    } else {
        tracing::debug!(
            "maintenance mode file path not found or not a regular file, using a default message"
        );
    }

    if body_content.is_empty() {
        body_content = [
            "<html><head><title>",
            status_code.as_str(),
            " ",
            status_code.canonical_reason().unwrap_or_default(),
            "</title></head><body><center><h1>",
            DEFAULT_BODY_CONTENT,
            "</h1></center></body></html>",
        ]
        .concat();
    }

    let mut body = Body::empty();
    let len = body_content.len() as u64;

    if !method.is_head() {
        body = Body::from(body_content)
    }

    let mut resp = Response::new(body);
    *resp.status_mut() = *status_code;
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    Ok(resp)
}
