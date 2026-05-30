// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module provides several HTTP security headers support.
//!

use http::header::{
    CONTENT_SECURITY_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use hyper::{Request, Response, header::HeaderValue};

use crate::body::Body;
use crate::{Error, handler::RequestHandlerOpts};

// Pre-computed static header values to avoid per-response parsing
static HSTS_VALUE: HeaderValue =
    HeaderValue::from_static("max-age=63072000; includeSubDomains; preload");
static XFO_VALUE: HeaderValue = HeaderValue::from_static("DENY");
static XCTO_VALUE: HeaderValue = HeaderValue::from_static("nosniff");
static CSP_VALUE: HeaderValue = HeaderValue::from_static("frame-ancestors 'self'");
static RP_VALUE: HeaderValue = HeaderValue::from_static("strict-origin-when-cross-origin");

pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.security_headers = enabled;
    tracing::info!(enabled, "security headers");
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

    // Referrer-Policy (RP)
    resp.headers_mut()
        .insert("referrer-policy", RP_VALUE.clone());
}

#[cfg(test)]
mod tests {
    use http::header::{
        CONTENT_SECURITY_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
    };
    use hyper::{Response, StatusCode};

    use super::append_headers;

    #[test]
    fn append_headers_sets_hsts() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        append_headers(&mut resp);
        let hsts = resp.headers().get(STRICT_TRANSPORT_SECURITY).unwrap();
        assert_eq!(
            hsts.to_str().unwrap(),
            "max-age=63072000; includeSubDomains; preload"
        );
    }

    #[test]
    fn append_headers_sets_x_frame_options() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        append_headers(&mut resp);
        let xfo = resp.headers().get(X_FRAME_OPTIONS).unwrap();
        assert_eq!(xfo.to_str().unwrap(), "DENY");
    }

    #[test]
    fn append_headers_sets_x_content_type_options() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        append_headers(&mut resp);
        let xcto = resp.headers().get(X_CONTENT_TYPE_OPTIONS).unwrap();
        assert_eq!(xcto.to_str().unwrap(), "nosniff");
    }

    #[test]
    fn append_headers_sets_csp() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        append_headers(&mut resp);
        let csp = resp.headers().get(CONTENT_SECURITY_POLICY).unwrap();
        assert_eq!(csp.to_str().unwrap(), "frame-ancestors 'self'");
    }

    #[test]
    fn append_headers_sets_all_four_headers() {
        let mut resp = Response::new(crate::body::empty());
        append_headers(&mut resp);
        assert!(resp.headers().contains_key(STRICT_TRANSPORT_SECURITY));
        assert!(resp.headers().contains_key(X_FRAME_OPTIONS));
        assert!(resp.headers().contains_key(X_CONTENT_TYPE_OPTIONS));
        assert!(resp.headers().contains_key(CONTENT_SECURITY_POLICY));
    }

    #[test]
    fn append_headers_sets_referrer_policy() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        append_headers(&mut resp);
        let rp = resp.headers().get("referrer-policy").unwrap();
        assert_eq!(rp.to_str().unwrap(), "strict-origin-when-cross-origin");
    }
}
