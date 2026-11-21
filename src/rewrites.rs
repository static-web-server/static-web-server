// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! `[[advanced.rewrites]]`: replaces the URI of a matching request with the rule's
//! destination and lets the pipeline serve it. Matching and destination building live in
//! [`crate::url_rules`].

use hyper::{Request, Response, Uri, header::HOST, http::HeaderValue};

use crate::body::Body;
use crate::{Error, Result, handler_opts::RequestHandlerOpts, url_rules};

/// Rewrites the request URI if a rule matches it. Returns `Some` only for the error page
/// of a destination that is not a valid URI.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &mut Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    let rewrites = opts.advanced_opts.as_ref()?.rewrites.as_deref()?;
    let rewrite = url_rules::find(rewrites, req)?;

    let uri = match rewrite
        .rule
        .resolve(req.uri())
        .and_then(|dest| merge_uris(req.uri(), &dest))
    {
        Ok(uri) => uri,
        Err(err) => {
            return url_rules::error_response(
                err.context("invalid rewrite destination"),
                opts,
                req,
            );
        }
    };
    tracing::trace!("uri matches rewrites glob pattern, rewriting to '{uri}'");
    *req.uri_mut() = uri;

    // RFC 9112 §3.2.2: keep `Host` in line with the authority, which may now target
    // another virtual host. `Host` has no user info, nor an empty port
    if let Some(authority) = req.uri().authority() {
        let host = match authority.port_u16() {
            Some(port) => format!("{}:{port}", authority.host()),
            None => authority.host().to_owned(),
        };
        if let Ok(host) = HeaderValue::from_str(&host) {
            req.headers_mut().insert(HOST, host);
        }
    }

    None
}

/// Parses `dest`, taking the scheme, authority, and path it lacks from `orig_uri`.
fn merge_uris(orig_uri: &Uri, dest: &str) -> Result<Uri> {
    let mut parts = dest.parse::<Uri>()?.into_parts();
    if parts.scheme.is_none() {
        parts.scheme = orig_uri.scheme().cloned();
    }
    if parts.authority.is_none() {
        parts.authority = orig_uri.authority().cloned();
    }
    if parts.path_and_query.is_none() {
        parts.path_and_query = orig_uri.path_and_query().cloned();
    }
    Ok(Uri::from_parts(parts)?)
}

#[cfg(test)]
mod tests {
    use super::pre_process;
    use crate::{
        handler_opts::RequestHandlerOpts,
        settings::{Advanced, Rewrites},
        url_rules::UrlRule,
    };
    use hyper::{Request, StatusCode, header::HOST};

    fn opts(rewrites: Option<Vec<Rewrites>>) -> RequestHandlerOpts {
        RequestHandlerOpts {
            advanced_opts: Some(Advanced {
                rewrites,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn rewrite(source: &str, dest: &str) -> Rewrites {
        Rewrites {
            rule: UrlRule::new(None, source, dest).unwrap(),
        }
    }

    fn rewritten(opts: &RequestHandlerOpts, uri: &str, host: &str) -> (String, String) {
        let mut req = Request::get(uri).header(HOST, host).body(()).unwrap();
        assert!(pre_process(opts, &mut req).is_none());
        let host = req.headers()[HOST].to_str().unwrap().to_owned();
        (req.uri().to_string(), host)
    }

    #[test]
    fn rewrites_none_configured_passes_through() {
        let mut req = Request::get("/").body(()).unwrap();
        assert!(pre_process(&RequestHandlerOpts::default(), &mut req).is_none());
        assert!(pre_process(&opts(None), &mut req).is_none());
        assert_eq!(req.uri(), "/");
    }

    #[test]
    fn rewrites_path_keeps_host() {
        let opts = opts(Some(vec![rewrite("/a/{*}", "/b/$1")]));
        assert_eq!(
            rewritten(&opts, "/a/x?q=1", "example.com"),
            ("/b/x?q=1".into(), "example.com".into())
        );
        assert_eq!(
            rewritten(&opts, "/c", "example.com"),
            ("/c".into(), "example.com".into())
        );
    }

    #[test]
    fn rewrites_absolute_destination_replaces_host() {
        let opts = opts(Some(vec![rewrite(
            "/{*}/{*}",
            "http://example.net:1234/d/$1?$2",
        )]));
        assert_eq!(
            rewritten(&opts, "/a/b?q=1", "example.com"),
            (
                "http://example.net:1234/d/a?b&q=1".into(),
                "example.net:1234".into()
            )
        );
    }

    #[test]
    fn rewrites_absolute_form_request_keeps_scheme_and_authority() {
        let opts = opts(Some(vec![rewrite("/a", "/b")]));
        assert_eq!(
            rewritten(&opts, "http://example.com:8080/a", "other.org"),
            (
                "http://example.com:8080/b".into(),
                "example.com:8080".into()
            ),
            "the authority of an absolute-form target replaces `Host`"
        );
    }

    #[test]
    fn rewrites_host_header_excludes_user_info() {
        let opts = opts(Some(vec![rewrite("/a", "http://user@other.example:81/b")]));
        assert_eq!(
            rewritten(&opts, "/a", "example.com"),
            (
                "http://user@other.example:81/b".into(),
                "other.example:81".into()
            )
        );
    }

    #[test]
    fn rewrites_host_header_omits_empty_port() {
        let opts = opts(Some(vec![rewrite("/a", "http://other.example:/b")]));
        assert_eq!(rewritten(&opts, "/a", "example.com").1, "other.example");
    }

    #[test]
    fn rewrites_invalid_destination_is_500() {
        let opts = opts(Some(vec![rewrite("/a", "http://[bad/")]));
        let mut req = Request::get("/a").body(()).unwrap();
        let resp = pre_process(&opts, &mut req).unwrap().unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(req.uri(), "/a", "uri must stay untouched on error");
    }
}
