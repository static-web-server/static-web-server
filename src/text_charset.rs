// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Declares a default `charset` parameter on `text/*` responses that don't
//! already have one. Similar in spirit to Apache's `AddDefaultCharset` and
//! nginx's `charset` directives, applied here to every `text/*` subtype.

use hyper::{
    Body, Response,
    header::{CONTENT_TYPE, HeaderValue},
};

use crate::{Error, handler::RequestHandlerOpts};

/// Validates a charset value against the RFC 7230 `token` grammar.
pub(crate) fn is_valid_charset(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

/// Adds `; charset=<value>` to the `Content-Type` header when the response
/// type is `text/*` and no `charset` parameter is already present. A no-op
/// when the option is disabled (empty value).
pub(crate) fn post_process(
    opts: &RequestHandlerOpts,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if opts.text_charset.is_empty() {
        return Ok(resp);
    }

    let ct = match resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
    {
        Some(v) if is_text(v) && !has_charset_param(v) => v,
        _ => return Ok(resp),
    };

    if let Ok(value) = HeaderValue::from_str(&format!("{ct}; charset={}", opts.text_charset)) {
        resp.headers_mut().insert(CONTENT_TYPE, value);
    }
    Ok(resp)
}

fn is_text(content_type: &str) -> bool {
    content_type
        .get(..5)
        .is_some_and(|p| p.eq_ignore_ascii_case("text/"))
}

fn has_charset_param(content_type: &str) -> bool {
    content_type.split(';').skip(1).any(|param| {
        param
            .trim_start()
            .get(..8)
            .is_some_and(|p| p.eq_ignore_ascii_case("charset="))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Body;

    fn opts(charset: &str) -> RequestHandlerOpts {
        RequestHandlerOpts {
            text_charset: charset.to_owned(),
            ..Default::default()
        }
    }

    fn resp_with(ct: &str) -> Response<Body> {
        let mut resp = Response::new(Body::empty());
        resp.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_str(ct).unwrap());
        resp
    }

    fn ct(resp: &Response<Body>) -> &str {
        resp.headers().get(CONTENT_TYPE).unwrap().to_str().unwrap()
    }

    #[test]
    fn annotates_bare_text_type() {
        let r = post_process(&opts("utf-8"), resp_with("text/plain")).unwrap();
        assert_eq!(ct(&r), "text/plain; charset=utf-8");
    }

    #[test]
    fn annotates_text_markdown() {
        let r = post_process(&opts("utf-8"), resp_with("text/markdown")).unwrap();
        assert_eq!(ct(&r), "text/markdown; charset=utf-8");
    }

    #[test]
    fn preserves_existing_charset() {
        let r = post_process(&opts("utf-8"), resp_with("text/html; charset=iso-8859-1")).unwrap();
        assert_eq!(ct(&r), "text/html; charset=iso-8859-1");
    }

    #[test]
    fn preserves_charset_with_unusual_casing_and_spacing() {
        let r = post_process(&opts("utf-8"), resp_with("text/html;CHARSET=utf-8")).unwrap();
        assert_eq!(ct(&r), "text/html;CHARSET=utf-8");
    }

    #[test]
    fn preserves_charset_when_not_first_param() {
        let r = post_process(
            &opts("utf-8"),
            resp_with("text/plain; foo=bar; charset=latin1"),
        )
        .unwrap();
        assert_eq!(ct(&r), "text/plain; foo=bar; charset=latin1");
    }

    #[test]
    fn ignores_non_text_type() {
        let r = post_process(&opts("utf-8"), resp_with("application/json")).unwrap();
        assert_eq!(ct(&r), "application/json");
    }

    #[test]
    fn ignores_when_disabled() {
        let r = post_process(&opts(""), resp_with("text/plain")).unwrap();
        assert_eq!(ct(&r), "text/plain");
    }

    #[test]
    fn ignores_when_content_type_missing() {
        let r = post_process(&opts("utf-8"), Response::new(Body::empty())).unwrap();
        assert!(r.headers().get(CONTENT_TYPE).is_none());
    }

    #[test]
    fn appends_when_unrelated_param_present() {
        let r = post_process(&opts("utf-8"), resp_with("text/plain; boundary=x")).unwrap();
        assert_eq!(ct(&r), "text/plain; boundary=x; charset=utf-8");
    }

    #[test]
    fn validates_charset_tokens() {
        assert!(is_valid_charset("utf-8"));
        assert!(is_valid_charset("ISO-8859-1"));
        assert!(is_valid_charset("windows-1252"));
        assert!(!is_valid_charset(""));
        assert!(!is_valid_charset("utf 8"));
        assert!(!is_valid_charset("utf\"8"));
    }
}
