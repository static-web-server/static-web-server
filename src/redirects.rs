// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Redirection module to handle config redirect URLs with pattern matching support.
//!
//! # Security: ReDoS / pattern complexity
//!
//! Redirect/rewrite source patterns are admin-supplied at startup. SWS
//! uses [`regex_lite`], which has **no backtracking** (linear-time NFA
//! engine), so the classic catastrophic-backtracking ReDoS class does
//! not apply. However:
//!
//! - Per-request work is still proportional to `pattern_size * uri_len`.
//!   To bound it, requests with URIs longer than [`MAX_URI_LEN_FOR_REGEX`]
//!   bytes are skipped (no regex evaluation, no redirect).
//! - Operators should treat redirect patterns as trusted configuration
//!   and avoid loading them from untrusted sources.

use headers::HeaderValue;
use hyper::{Request, Response, StatusCode};
use regex_lite::Regex;

use crate::body::Body;
use crate::{Error, error_page, handler::RequestHandlerOpts, settings::Redirects};

/// Maximum URI length (bytes) that will be fed to the redirect regex
/// engine. Requests above this size skip redirect matching entirely.
///
/// 8 KiB matches the common HTTP-server URI cap and is more than the
/// largest realistic redirect source while still bounding per-request
/// regex work to a small constant.
pub(crate) const MAX_URI_LEN_FOR_REGEX: usize = 8 * 1024;

/// Applies redirect rules to a request if necessary.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    let redirects = opts.advanced_opts.as_ref()?.redirects.as_deref()?;

    let uri = req.uri();
    let uri_path = uri.path();
    // SECURITY (ReDoS bound): refuse to run any regex against
    // unreasonably long URIs. See module-level docs.
    if uri_path.len() > MAX_URI_LEN_FOR_REGEX {
        tracing::debug!(
            "redirects: skipping match, uri path length {} exceeds cap {}",
            uri_path.len(),
            MAX_URI_LEN_FOR_REGEX
        );
        return None;
    }
    let host = req
        .headers()
        .get(http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let mut uri_host = uri.host().unwrap_or(host).to_owned();
    if let Some(uri_port) = uri.port_u16() {
        uri_host.push_str(&format!(":{uri_port}"));
    }
    let matched = get_redirection(&uri_host, uri_path, Some(redirects))?;
    let mut dest = match replace_placeholders(
        uri_path,
        &matched.source,
        &matched.destination,
        &matched.replacer,
    ) {
        Ok(dest) => dest,
        Err(err) => return handle_error(err, opts, req),
    };

    // Preserve the client's query string across the redirect
    // in an Apache's QSA rewrite option fashion.
    if let Some(query) = uri.query() {
        if !dest.ends_with('?') && !dest.ends_with('&') {
            dest.push(if dest.contains('?') { '&' } else { '?' });
        }
        dest.push_str(query);
    }

    match HeaderValue::from_str(&dest) {
        Ok(loc) => {
            let mut resp = Response::new(crate::body::empty());
            resp.headers_mut().insert(hyper::header::LOCATION, loc);
            *resp.status_mut() = matched.kind;
            tracing::trace!(
                "uri matches redirects glob pattern, redirecting with status '{}'",
                matched.kind
            );
            Some(Ok(resp))
        }
        Err(err) => handle_error(
            Error::new(err).context("invalid header value from current uri"),
            opts,
            req,
        ),
    }
}

/// Replaces placeholders in the destination URI by matching capture groups from the original URI.
#[doc(hidden)]
pub fn replace_placeholders(
    orig_uri: &str,
    regex: &Regex,
    dest_uri: &str,
    ac: &aho_corasick::AhoCorasick,
) -> Result<String, Error> {
    let regex_caps = if let Some(regex_caps) = regex.captures(orig_uri) {
        regex_caps
    } else {
        return Err(Error::msg("regex didn't match, extracting captures failed"));
    };

    let caps: Vec<&str> = (0..regex_caps.len())
        .map(|i| regex_caps.get(i).map(|s| s.as_str()).unwrap_or(""))
        .collect();

    tracing::debug!("url redirects/rewrites regex equivalent: {regex}");
    tracing::debug!("url redirects/rewrites glob pattern captures: {caps:?}");
    tracing::debug!("url redirects/rewrites glob pattern destination: {dest_uri:?}");

    match ac.try_replace_all(dest_uri, &caps) {
        Ok(dest) => {
            tracing::debug!("url redirects/rewrites glob pattern destination replaced: {dest:?}");
            Ok(dest)
        }
        Err(err) => Err(Error::new(err).context("failed replacing captures")),
    }
}

/// Logs error and produces an Internal Server Error response.
pub(crate) fn handle_error<T>(
    err: Error,
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    tracing::error!("{err:?}");
    Some(error_page::error_response(
        req.uri(),
        req.method(),
        &StatusCode::INTERNAL_SERVER_ERROR,
        &opts.page404,
        &opts.page50x,
    ))
}

/// It returns a redirect's destination path and status code if the current request uri
/// matches against the provided redirect's array.
pub fn get_redirection<'a>(
    uri_host: &'a str,
    uri_path: &'a str,
    redirects_opts: Option<&'a [Redirects]>,
) -> Option<&'a Redirects> {
    if let Some(redirects_vec) = redirects_opts {
        for redirect_entry in redirects_vec {
            // Match `host` redirect against `uri_host` if specified
            if let Some(host) = &redirect_entry.host {
                tracing::debug!(
                    "checking host '{host}' redirect entry against uri host '{uri_host}'"
                );
                if !host.eq(uri_host) {
                    continue;
                }
            }

            // Match source glob pattern against the request uri path
            if redirect_entry.source.is_match(uri_path) {
                return Some(redirect_entry);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::pre_process;
    use crate::body::Body;
    use crate::{
        Error,
        handler::RequestHandlerOpts,
        settings::{Advanced, Redirects, build_placeholder_replacer},
    };
    use hyper::{Request, Response, StatusCode};
    use regex_lite::Regex;

    fn make_request(host: &str, uri: &str) -> Request<Body> {
        let mut builder = Request::builder();
        if !host.is_empty() {
            builder = builder.header("Host", host);
        }
        builder
            .method("GET")
            .uri(uri)
            .body(crate::body::empty())
            .unwrap()
    }

    fn get_redirects() -> Vec<Redirects> {
        let s1 = Regex::new(r"/source1$").unwrap();
        let r1 = build_placeholder_replacer(&s1);
        let s2 = Regex::new(r"/source2$").unwrap();
        let r2 = build_placeholder_replacer(&s2);
        let s3 = Regex::new(r"/(prefix/)?(source3)/(.*)").unwrap();
        let r3 = build_placeholder_replacer(&s3);
        let s4 = Regex::new(r"/source4/(.*)").unwrap();
        let r4 = build_placeholder_replacer(&s4);
        vec![
            Redirects {
                host: None,
                source: s1,
                destination: "/destination1".into(),
                kind: StatusCode::FOUND,
                replacer: r1,
            },
            Redirects {
                host: Some("example.com".into()),
                source: s2,
                destination: "/destination2".into(),
                kind: StatusCode::MOVED_PERMANENTLY,
                replacer: r2,
            },
            Redirects {
                host: Some("example.info".into()),
                source: s3,
                destination: "/destination3/$2/$3".into(),
                kind: StatusCode::MOVED_PERMANENTLY,
                replacer: r3,
            },
            Redirects {
                host: None,
                source: s4,
                destination: "/destination4?p=$1".into(),
                kind: StatusCode::FOUND,
                replacer: r4,
            },
        ]
    }

    fn is_redirect(result: Option<Result<Response<Body>, Error>>) -> Option<(StatusCode, String)> {
        if let Some(Ok(response)) = result {
            let location = response.headers().get("Location")?.to_str().unwrap().into();
            Some((response.status(), location))
        } else {
            None
        }
    }

    #[test]
    fn test_no_redirects() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    advanced_opts: None,
                    ..Default::default()
                },
                &make_request("", "/")
            )
            .is_none()
        );

        assert!(
            pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("", "/")
            )
            .is_none()
        );
    }

    #[test]
    fn test_no_match() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("example.com", "/source2/whatever")
            )
            .is_none()
        );

        assert!(
            pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("", "/source2")
            )
            .is_none()
        );
    }

    #[test]
    fn test_match() {
        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("", "/source1")
            )),
            Some((StatusCode::FOUND, "/destination1".into()))
        );

        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("example.com", "/source2")
            )),
            Some((StatusCode::MOVED_PERMANENTLY, "/destination2".into()))
        );

        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("example.info", "/source3/whatever")
            )),
            Some((
                StatusCode::MOVED_PERMANENTLY,
                "/destination3/source3/whatever".into()
            ))
        );

        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("", "/source4/whatever")
            )),
            Some((StatusCode::FOUND, "/destination4?p=whatever".into()))
        );
    }

    #[test]
    fn test_query() {
        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("", "/source1?q=query-string")
            )),
            Some((StatusCode::FOUND, "/destination1?q=query-string".into()))
        );

        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("example.com", "/source2?q=query-string")
            )),
            Some((
                StatusCode::MOVED_PERMANENTLY,
                "/destination2?q=query-string".into()
            ))
        );

        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("example.info", "/source3/whatever?q=query-string")
            )),
            Some((
                StatusCode::MOVED_PERMANENTLY,
                "/destination3/source3/whatever?q=query-string".into()
            ))
        );

        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        redirects: Some(get_redirects()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &make_request("", "/source4/whatever?q=query-string")
            )),
            Some((
                StatusCode::FOUND,
                "/destination4?p=whatever&q=query-string".into()
            ))
        );
    }

    // Property-based regression tests for `replace_placeholders` and the
    // upstream URI length guard.
    //
    // The guard exists so adversarial inputs cannot pin the CPU on
    // regex matching; the property is "calls never panic and respect
    // the cap". `replace_placeholders` itself must also be total over
    // any byte-length input bounded by `MAX_URI_LEN_FOR_REGEX`.
    use super::{MAX_URI_LEN_FOR_REGEX, replace_placeholders};
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128, ..ProptestConfig::default()
        })]

        /// `replace_placeholders` must never panic on arbitrary input
        /// pairs (orig URI, destination template) within the URI cap,
        /// regardless of whether the regex matches.
        #[test]
        fn prop_replace_placeholders_never_panics(
            orig in "\\PC{0,512}",
            dest in "\\PC{0,512}",
        ) {
            // A representative source pattern with 5 capture groups
            // \u2014 mirrors realistic redirect rules.
            let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
            let ac = build_placeholder_replacer(&re);
            let _ = replace_placeholders(&orig, &re, &dest, &ac);
        }

        /// When the regex does NOT match, `replace_placeholders` MUST
        /// return an `Err` (no silent fallthrough).
        #[test]
        fn prop_replace_placeholders_no_match_returns_err(
            // A leading char that prevents the regex from anchoring.
            tail in "[a-zA-Z0-9_.-]{0,64}",
            dest in "\\PC{0,64}",
        ) {
            let orig = format!("no-leading-slash-{tail}");
            let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
            let ac = build_placeholder_replacer(&re);
            prop_assert!(replace_placeholders(&orig, &re, &dest, &ac).is_err());
        }

        /// On a successful match, the produced destination MUST contain
        /// only the captured substrings (or original literals) — i.e.
        /// no `$N` placeholders survive for `N < captures_len()`.
        #[test]
        fn prop_replace_placeholders_substitutes_all_indices(
            a in "[a-zA-Z0-9]{1,16}",
            b in "[a-zA-Z0-9]{1,16}",
            c in "[a-zA-Z0-9]{1,16}",
            d in "[a-zA-Z0-9]{1,16}",
            e in "[a-zA-Z0-9]{1,16}",
        ) {
            let orig = format!("/{a}/{b}/{c}/{d}/{e}");
            let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
            let ac = build_placeholder_replacer(&re);
            let dest = "/$0|$1|$2|$3|$4|$5".to_string();
            let out = replace_placeholders(&orig, &re, &dest, &ac).unwrap();
            // $0 = whole match (orig), then capture groups 1..=5.
            let expected = format!("/{orig}|{a}|{b}|{c}|{d}|{e}");
            prop_assert_eq!(out, expected);
        }
    }

    /// `MAX_URI_LEN_FOR_REGEX` is a security/perf invariant; this test
    /// keeps it as a tripwire if anyone ever lowers it accidentally.
    #[test]
    fn max_uri_len_for_regex_is_at_least_8kib() {
        const { assert!(MAX_URI_LEN_FOR_REGEX >= 8 * 1024) };
    }
}
