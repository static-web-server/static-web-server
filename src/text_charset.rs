// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Declares a default `charset` parameter on `text/*` responses that don't
//! already have one. Similar in spirit to Apache's `AddDefaultCharset` and
//! nginx's `charset` directives, applied here to every `text/*` subtype.

use hyper::{
    Response,
    header::{CONTENT_TYPE, HeaderValue},
};
use mime_guess::{Mime, mime};

use crate::body::Body;
use crate::exts::mime::MimeExt;
use crate::{Error, handler::RequestHandlerOpts};

/// MIME types requiring an explicit `; charset=utf-8` header.
///
/// These mime types contain human-readable text but do not default to UTF-8.
/// Forcing UTF-8 prevents broken symbols and accents on the client side.
/// It also overrides obsolete fallbacks like US-ASCII or ISO-8859-1.
const TEXT_ONLY_TYPES: [&str; 8] = [
    "application/atom+xml",
    "application/rss+xml",
    "text/calendar",
    "text/csv",
    "text/html",
    "text/markdown",
    "text/plain",
    "text/xml",
];

/// Adds `; charset=utf-8` to the `Content-Type` header when the response
/// type matches the  predefined list of `text` mime types and
/// no `charset` parameter is already present.
pub(crate) fn post_process(
    opts: &RequestHandlerOpts,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if !opts.text_charset {
        return Ok(resp);
    }

    let new_header = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<Mime>().ok().map(|m| (s, m)))
        .filter(|(_, m)| {
            m.contains_text_only(&TEXT_ONLY_TYPES) && m.get_param(mime::CHARSET).is_none()
        })
        .map(|(s, _)| format!("{}; charset=utf-8", s.trim_end().trim_end_matches(';')))
        .and_then(|s| HeaderValue::from_str(&s).ok());

    if let Some(value) = new_header {
        resp.headers_mut().insert(CONTENT_TYPE, value);
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body;

    fn opts(text_charset: bool) -> RequestHandlerOpts {
        RequestHandlerOpts {
            text_charset,
            ..Default::default()
        }
    }

    fn resp_with(ct: &str) -> Response<Body> {
        let mut resp = Response::new(body::empty());
        resp.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_str(ct).unwrap());
        resp
    }

    fn ct(resp: &Response<Body>) -> &str {
        resp.headers().get(CONTENT_TYPE).unwrap().to_str().unwrap()
    }

    #[test]
    fn annotates_bare_text_type() {
        let r = post_process(&opts(true), resp_with("text/plain")).unwrap();
        assert_eq!(ct(&r), "text/plain; charset=utf-8");
    }

    #[test]
    fn annotates_text_markdown() {
        let r = post_process(&opts(true), resp_with("text/markdown")).unwrap();
        assert_eq!(ct(&r), "text/markdown; charset=utf-8");
    }

    #[test]
    fn preserves_existing_charset() {
        let r = post_process(&opts(true), resp_with("text/html; charset=iso-8859-1")).unwrap();
        assert_eq!(ct(&r), "text/html; charset=iso-8859-1");
    }

    #[test]
    fn preserves_charset_with_unusual_casing_and_spacing() {
        let r = post_process(&opts(true), resp_with("text/html;CHARSET=utf-8")).unwrap();
        assert_eq!(ct(&r), "text/html;CHARSET=utf-8");
    }

    #[test]
    fn preserves_charset_when_not_first_param() {
        let r = post_process(
            &opts(true),
            resp_with("text/plain; foo=bar; charset=latin1"),
        )
        .unwrap();
        assert_eq!(ct(&r), "text/plain; foo=bar; charset=latin1");
    }

    #[test]
    fn ignores_non_text_type() {
        let r = post_process(&opts(true), resp_with("application/json")).unwrap();
        assert_eq!(ct(&r), "application/json");
    }

    #[test]
    fn ignores_when_disabled() {
        let r = post_process(&opts(false), resp_with("text/plain")).unwrap();
        assert_eq!(ct(&r), "text/plain");
    }

    #[test]
    fn ignores_when_content_type_missing() {
        let r = post_process(&opts(true), Response::new(body::empty())).unwrap();
        assert!(r.headers().get(CONTENT_TYPE).is_none());
    }

    #[test]
    fn appends_when_unrelated_param_present() {
        let r = post_process(&opts(true), resp_with("text/plain; boundary=x")).unwrap();
        assert_eq!(ct(&r), "text/plain; boundary=x; charset=utf-8");
    }
}
