// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! `[[advanced.redirects]]`: answers a matching request with a 301 or 302 to the rule's
//! destination. Matching and destination building live in [`crate::url_rules`].

use hyper::{Request, Response, StatusCode, header::LOCATION, http::HeaderValue};

use crate::body::Body;
use crate::url_rules::{self, RuleMatch};
use crate::{Error, handler_opts::RequestHandlerOpts};

/// Answers with a redirect if a rule matches the request, or `414` when its path is too
/// long to be matched.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    let redirects = opts.advanced_opts.as_ref()?.redirects.as_deref()?;
    let (redirect, dest) = match url_rules::find(redirects, req) {
        RuleMatch::NoRule => return None,
        RuleMatch::Rule(redirect, dest) => (redirect, dest),
        RuleMatch::PathTooLong => {
            return url_rules::status_response(StatusCode::URI_TOO_LONG, opts, req);
        }
    };

    let loc = match dest.and_then(|dest| HeaderValue::try_from(dest).map_err(Error::from)) {
        Ok(loc) => loc,
        Err(err) => {
            return url_rules::error_response(
                err.context("invalid redirect destination"),
                opts,
                req,
            );
        }
    };

    tracing::trace!(
        "uri matches redirects glob pattern, redirecting with status '{}'",
        redirect.kind
    );
    let mut resp = Response::new(crate::body::empty());
    resp.headers_mut().insert(LOCATION, loc);
    *resp.status_mut() = redirect.kind;
    Some(Ok(resp))
}

#[cfg(test)]
mod tests {
    use super::pre_process;
    use crate::{
        handler_opts::RequestHandlerOpts,
        settings::{Advanced, Redirects},
        url_rules::{MAX_URI_LEN_FOR_REGEX, UrlRule},
    };
    use hyper::{Request, StatusCode};

    fn opts(redirects: Option<Vec<Redirects>>) -> RequestHandlerOpts {
        RequestHandlerOpts {
            advanced_opts: Some(Advanced {
                redirects,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn redirect(host: Option<&str>, source: &str, dest: &str, kind: StatusCode) -> Redirects {
        Redirects {
            rule: UrlRule::new(host, source, dest).unwrap(),
            kind,
        }
    }

    fn location(opts: &RequestHandlerOpts, uri: &str) -> Option<(StatusCode, String)> {
        let req = Request::get(uri).body(()).unwrap();
        let resp = pre_process(opts, &req)?.unwrap();
        let loc = resp.headers().get("location")?.to_str().unwrap().to_owned();
        Some((resp.status(), loc))
    }

    #[test]
    fn redirects_none_configured_passes_through() {
        let req = Request::get("/").body(()).unwrap();
        assert!(pre_process(&RequestHandlerOpts::default(), &req).is_none());
        assert!(pre_process(&opts(None), &req).is_none());
        assert!(pre_process(&opts(Some(vec![])), &req).is_none());
    }

    #[test]
    fn redirects_status_and_location() {
        let opts = opts(Some(vec![
            redirect(None, "/a", "/b", StatusCode::MOVED_PERMANENTLY),
            redirect(None, "/c", "https://x.example/d", StatusCode::FOUND),
        ]));
        assert_eq!(
            location(&opts, "/a"),
            Some((StatusCode::MOVED_PERMANENTLY, "/b".into()))
        );
        assert_eq!(
            location(&opts, "/c?q=1"),
            Some((StatusCode::FOUND, "https://x.example/d?q=1".into()))
        );
        assert_eq!(location(&opts, "/e"), None);
    }

    #[test]
    fn redirects_paths_longer_than_the_cap_are_414() {
        let configured = opts(Some(vec![redirect(
            None,
            "/a/{**}",
            "/b/$1",
            StatusCode::FOUND,
        )]));
        let at_cap = ["/a/", &"x".repeat(MAX_URI_LEN_FOR_REGEX - 3)].concat();
        assert_eq!(
            location(&configured, &at_cap),
            Some((StatusCode::FOUND, at_cap.replacen("/a/", "/b/", 1)))
        );
        let req = Request::get([&at_cap, "x"].concat()).body(()).unwrap();
        let resp = pre_process(&configured, &req).unwrap().unwrap();
        assert_eq!(resp.status(), StatusCode::URI_TOO_LONG);
        assert!(resp.headers().get("location").is_none());

        let req = Request::get([&at_cap, "x"].concat()).body(()).unwrap();
        assert!(pre_process(&opts(Some(vec![])), &req).is_none());
    }

    #[test]
    fn redirects_invalid_location_is_500() {
        // Startup rejects such a destination, so build the rule by hand
        let mut invalid = redirect(None, "/a", "/b", StatusCode::FOUND);
        invalid.rule.destination = "/b\nc".into();
        let opts = opts(Some(vec![invalid]));
        let req = Request::get("/a").body(()).unwrap();
        let resp = pre_process(&opts, &req).unwrap().unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(resp.headers().get("location").is_none());
    }
}
