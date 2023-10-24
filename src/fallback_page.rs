// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Fallback page module useful for a custom page default.
//!

use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Response, StatusCode};
use mime_guess::mime;

/// Checks if a fallback response can be generated, i.e. if it is a `GET` request
/// that would result in a `404` error and a fallback page is configured.
/// If a response can be generated then is returned otherwise `None`.
pub fn fallback_response(page_fallback: &[u8]) -> Response<Body> {
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
