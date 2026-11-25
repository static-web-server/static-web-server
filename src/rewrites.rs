// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! `[[advanced.rewrites]]`: replaces the URI of a matching request with the rule's
//! destination and lets the pipeline serve it. Matching and destination building live in
//! [`crate::url_rules`].

use hyper::{Request, Response, Uri, header::HOST, http::HeaderValue};
use std::path::Path;

use crate::body::Body;
use crate::{
    Error, Result,
    fs::path::sanitize_path,
    handler_opts::RequestHandlerOpts,
    settings::Rewrites,
    static_files::RewrittenFrom,
    url_rules::{self, RequestHost},
};

/// Request URI and `Host` as the client sent them, kept in the request extensions when a
/// rewrite replaces them.
#[derive(Clone)]
struct ClientRequest {
    uri: Uri,
    host: Option<HeaderValue>,
}

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
    let client = ClientRequest {
        host: req.headers().get(HOST).cloned(),
        uri: std::mem::replace(req.uri_mut(), uri),
    };
    req.extensions_mut().insert(client);

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

/// Returns the request URI and `Host` as the client sent them, along with the redirect
/// and rewrite rules, when [`pre_process`] rewrote the request. `None` otherwise.
pub(crate) fn rewritten_from<'a, T>(
    opts: &'a RequestHandlerOpts,
    req: &'a Request<T>,
) -> Option<RewrittenFrom<'a>> {
    let advanced = opts.advanced_opts.as_ref()?;
    let client = req.extensions().get::<ClientRequest>()?;
    Some(RewrittenFrom {
        uri: &client.uri,
        host: client.host.as_ref(),
        rewrites: advanced.rewrites.as_deref()?,
        redirects: advanced.redirects.as_deref().unwrap_or_default(),
    })
}

/// Returns `true` when the client's path with `/` appended matches no redirect rule and
/// rewrites to the same host and directory as the client's path, with a trailing slash,
/// so a trailing-slash redirect to it is served without a further redirect.
pub(crate) fn has_same_destination_with_slash(from: &RewrittenFrom<'_>) -> bool {
    let host = RequestHost::new(from.uri, from.host);
    let uri_path = from.uri.path();
    let slashed = [uri_path, "/"].concat();
    if url_rules::find_for(from.redirects, host.as_ref(), &slashed).is_some() {
        return false;
    }
    let (Some(dest), Some(dest_slashed)) = (
        internal_destination(from.rewrites, host.as_ref(), uri_path),
        internal_destination(from.rewrites, host.as_ref(), &slashed),
    ) else {
        return false;
    };
    dest_slashed.path().ends_with('/')
        && dest.authority() == dest_slashed.authority()
        && sanitize_path(Path::new(""), dest.path())
            == sanitize_path(Path::new(""), dest_slashed.path())
}

/// Returns the destination of the rewrite that applies to a request for `uri_path`
/// addressed to `host`, `None` when no rule applies.
fn internal_destination(
    rules: &[Rewrites],
    host: Option<&RequestHost<'_>>,
    uri_path: &str,
) -> Option<Uri> {
    let rule = &url_rules::find_for(rules, host, uri_path)?.rule;
    let dest =
        url_rules::replace_placeholders(uri_path, &rule.source, &rule.destination, &rule.replacer)
            .ok()?;
    dest.parse().ok()
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
    use super::{has_same_destination_with_slash, pre_process, rewritten_from};
    use crate::{
        handler_opts::RequestHandlerOpts,
        settings::{Advanced, Redirects, Rewrites},
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

    fn host_rewrite(host: &str, source: &str, dest: &str) -> Rewrites {
        Rewrites {
            rule: UrlRule::new(Some(host), source, dest).unwrap(),
        }
    }

    /// Rewrites a request for `uri` sent to `host`, and returns whether its path with a
    /// slash has the same destination.
    fn same_destination_with_slash(rules: Vec<Rewrites>, uri: &str, host: &str) -> bool {
        let opts = opts(Some(rules));
        let mut req = Request::get(uri).header(HOST, host).body(()).unwrap();
        assert!(pre_process(&opts, &mut req).is_none());
        let from = rewritten_from(&opts, &req).expect("a rule must rewrite the request");
        has_same_destination_with_slash(&from)
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

    #[test]
    fn rewrites_not_applied_has_no_client_request() {
        let opts = opts(Some(vec![rewrite("/a", "/b")]));
        let mut req = Request::get("/c").body(()).unwrap();
        assert!(pre_process(&opts, &mut req).is_none());
        assert!(rewritten_from(&opts, &req).is_none());
    }

    #[test]
    fn rewrites_same_destination_with_slash_for_prefix_rule() {
        let rules = || vec![rewrite("/prefix{**}", "/$1")];
        assert!(same_destination_with_slash(rules(), "/prefix", "localhost"));
        assert!(same_destination_with_slash(
            rules(),
            "/prefix/dir",
            "localhost"
        ));
    }

    #[test]
    fn rewrites_other_destination_with_slash_for_exact_rule() {
        let rules = vec![rewrite("/exact", "/dir/"), rewrite("/exact/", "/other/")];
        assert!(!same_destination_with_slash(rules, "/exact", "localhost"));

        let rules = vec![rewrite("/exact", "/dir/")];
        assert!(!same_destination_with_slash(rules, "/exact", "localhost"));
    }

    #[test]
    fn rewrites_other_destination_with_slash_without_trailing_slash() {
        let rules = vec![rewrite("/app/{**}", "/dir")];
        assert!(!same_destination_with_slash(rules, "/app/x", "localhost"));
    }

    #[test]
    fn rewrites_other_destination_with_slash_for_other_host() {
        let rules = vec![
            rewrite("/a", "/dir"),
            rewrite("/a/", "http://other.local/dir/"),
        ];
        assert!(!same_destination_with_slash(rules, "/a", "localhost"));
    }

    #[test]
    fn rewrites_same_destination_with_slash_for_host_rule() {
        let rules = || vec![host_rewrite("a.example", "/a{**}", "/dir$1")];
        assert!(same_destination_with_slash(rules(), "/a", "a.example"));
        assert!(same_destination_with_slash(rules(), "/a", "A.Example:8080"));
        assert!(same_destination_with_slash(
            rules(),
            "http://a.example/a",
            "b.example"
        ));
    }

    #[test]
    fn rewrites_other_destination_with_slash_for_redirect_rule() {
        let rules = || vec![rewrite("/a{**}", "/dir$1")];
        let redirect = |host| Redirects {
            rule: UrlRule::new(host, "/a/", "/elsewhere").unwrap(),
            kind: StatusCode::MOVED_PERMANENTLY,
        };
        let same_destination = |redirect, host| {
            let mut opts = opts(Some(rules()));
            if let Some(advanced) = opts.advanced_opts.as_mut() {
                advanced.redirects = Some(vec![redirect]);
            }
            let mut req = Request::get("/a").header(HOST, host).body(()).unwrap();
            assert!(pre_process(&opts, &mut req).is_none());
            has_same_destination_with_slash(&rewritten_from(&opts, &req).unwrap())
        };
        assert!(!same_destination(redirect(None), "localhost"));
        assert!(!same_destination(redirect(Some("a.example")), "a.example"));
        assert!(
            same_destination(redirect(Some("a.example")), "b.example"),
            "a redirect rule of another host does not apply"
        );
    }

    #[test]
    fn rewrites_same_destination_with_slash_matches_client_host() {
        // The rewrite replaces `Host`, and the rule only applies to the one the client sent
        let rules = vec![host_rewrite(
            "a.example",
            "/a{**}",
            "http://other.example/dir$1",
        )];
        assert!(same_destination_with_slash(rules, "/a", "a.example"));
    }

    #[test]
    fn rewrites_other_destination_with_slash_for_rule_of_other_host() {
        let rules = vec![
            host_rewrite("a.example", "/a", "/dir"),
            host_rewrite("b.example", "/a/", "/dir/"),
        ];
        assert!(!same_destination_with_slash(rules, "/a", "a.example"));
    }
}
