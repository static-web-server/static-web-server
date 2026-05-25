// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module provides several HTTP security headers support.
//!

use http::header::{
    CONTENT_SECURITY_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use hyper::{Body, Request, Response, header::HeaderValue};

use crate::{Error, handler::RequestHandlerOpts};

// Pre-computed static header values to avoid per-response parsing
static HSTS_VALUE: HeaderValue =
    HeaderValue::from_static("max-age=63072000; includeSubDomains; preload");
static XFO_VALUE: HeaderValue = HeaderValue::from_static("DENY");
static XCTO_VALUE: HeaderValue = HeaderValue::from_static("nosniff");
static CSP_VALUE: HeaderValue = HeaderValue::from_static("frame-ancestors 'self'");

pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.security_headers = enabled;
    tracing::info!("security headers: enabled={enabled}");
}

/// Appends security headers to a response if necessary
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    _req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if opts.security_headers {
        append_headers(&mut resp);
    }
    Ok(resp)
}

/// It appends security headers like `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload` (2 years max-age),
///`X-Frame-Options: DENY` and `Content-Security-Policy: frame-ancestors 'self'`.
pub fn append_headers(resp: &mut Response<Body>) {
    // Strict-Transport-Security (HSTS)
    resp.headers_mut()
        .insert(STRICT_TRANSPORT_SECURITY, HSTS_VALUE.clone());

    // X-Frame-Options
    resp.headers_mut()
        .insert(X_FRAME_OPTIONS, XFO_VALUE.clone());

    // X-Content-Type-Options
    resp.headers_mut()
        .insert(X_CONTENT_TYPE_OPTIONS, XCTO_VALUE.clone());

    // Content Security Policy (CSP)
    resp.headers_mut()
        .insert(CONTENT_SECURITY_POLICY, CSP_VALUE.clone());
}
