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

use crate::{Error, handler::RequestHandlerOpts, settings::cli::General};

// Pre-computed static header values to avoid per-response parsing
static HSTS_VALUE: HeaderValue =
    HeaderValue::from_static("max-age=63072000; includeSubDomains; preload");
static XFO_VALUE: HeaderValue = HeaderValue::from_static("DENY");
static XCTO_VALUE: HeaderValue = HeaderValue::from_static("nosniff");
static CSP_VALUE: HeaderValue = HeaderValue::from_static("frame-ancestors 'self'");

/// Whether the server uses TLS (`--http2`), which `Strict-Transport-Security` requires.
pub(crate) fn tls_enabled(general: &General) -> bool {
    #[cfg(feature = "http2")]
    return general.http2;
    #[cfg(not(feature = "http2"))]
    {
        let _ = general;
        false
    }
}

/// Enable the security headers.
/// `Strict-Transport-Security` is only sent when the server uses TLS (`--http2`).
pub(crate) fn init(enabled: bool, tls: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.security_headers = enabled;
    handler_opts.hsts = enabled && tls;
    tracing::info!(
        "security headers: enabled={enabled}, hsts={}",
        handler_opts.hsts
    );
}

/// Appends security headers to a response if necessary
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    _req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if opts.security_headers {
        if opts.hsts {
            append_hsts_header(&mut resp);
        }
        append_common_headers(&mut resp);
    }
    Ok(resp)
}

/// It appends security headers like `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload` (2 years max-age),
///`X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff` and `Content-Security-Policy: frame-ancestors 'self'`.
///
/// RFC 6797 forbids sending `Strict-Transport-Security` over plain HTTP,
/// so only use this function for responses sent over TLS.
pub fn append_headers(resp: &mut Response<Body>) {
    append_hsts_header(resp);
    append_common_headers(resp);
}

/// It appends the `Strict-Transport-Security` (HSTS) header.
fn append_hsts_header(resp: &mut Response<Body>) {
    resp.headers_mut()
        .insert(STRICT_TRANSPORT_SECURITY, HSTS_VALUE.clone());
}

/// It appends the security headers that do not depend on the connection transport.
fn append_common_headers(resp: &mut Response<Body>) {
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

#[cfg(test)]
mod tests {
    use http::header::{
        CONTENT_SECURITY_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
    };
    use hyper::{Body, Request, Response};

    use super::{append_headers, init, post_process};
    use crate::handler::RequestHandlerOpts;

    fn assert_common_headers(resp: &Response<Body>) {
        let headers = resp.headers();
        assert_eq!(headers.get(X_FRAME_OPTIONS).unwrap(), "DENY");
        assert_eq!(headers.get(X_CONTENT_TYPE_OPTIONS).unwrap(), "nosniff");
        assert_eq!(
            headers.get(CONTENT_SECURITY_POLICY).unwrap(),
            "frame-ancestors 'self'"
        );
    }

    #[test]
    fn init_enables_hsts_only_with_tls() {
        for (enabled, tls, hsts) in [
            (true, true, true),
            (true, false, false),
            (false, true, false),
            (false, false, false),
        ] {
            let mut opts = RequestHandlerOpts::default();
            init(enabled, tls, &mut opts);
            assert_eq!(opts.security_headers, enabled);
            assert_eq!(opts.hsts, hsts, "enabled={enabled} tls={tls}");
        }
    }

    #[test]
    fn post_process_sends_hsts_only_with_tls() {
        for (enabled, tls) in [(true, true), (true, false), (false, true), (false, false)] {
            let mut opts = RequestHandlerOpts::default();
            init(enabled, tls, &mut opts);
            let req = Request::new(Body::empty());
            let resp = post_process(&opts, &req, Response::new(Body::empty())).unwrap();

            if enabled {
                assert_common_headers(&resp);
            } else {
                assert!(resp.headers().is_empty(), "tls={tls}");
            }
            assert_eq!(
                resp.headers().get(STRICT_TRANSPORT_SECURITY).is_some(),
                enabled && tls,
                "enabled={enabled} tls={tls}"
            );
        }
    }

    #[test]
    fn append_headers_sets_all_headers() {
        let mut resp = Response::new(Body::empty());
        append_headers(&mut resp);
        assert_eq!(
            resp.headers().get(STRICT_TRANSPORT_SECURITY).unwrap(),
            "max-age=63072000; includeSubDomains; preload"
        );
        assert_common_headers(&resp);
    }
}
