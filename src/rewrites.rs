// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that allows to rewrite request URLs with pattern matching support.
//!

use headers::HeaderValue;
use hyper::{header::HOST, Body, Request, Response, StatusCode, Uri};

use crate::{
    handler::RequestHandlerOpts,
    redirects::{handle_error, replace_placeholders},
    settings::{file::RedirectsKind, Rewrites},
    Error,
};

/// Applies rewrite rules to a request if necessary.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &mut Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    let rewrites = opts.advanced_opts.as_ref()?.rewrites.as_deref()?;
    let uri_path = req.uri().path();

    let matched = rewrite_uri_path(uri_path, Some(rewrites))?;
    let dest = match replace_placeholders(uri_path, &matched.source, &matched.destination) {
        Ok(dest) => dest,
        Err(err) => return handle_error(err, opts, req),
    };

    if let Some(redirect_type) = &matched.redirect {
        // Handle redirects
        let loc = match HeaderValue::from_str(&dest) {
            Ok(val) => val,
            Err(err) => {
                return handle_error(
                    Error::new(err).context("invalid header value from current uri"),
                    opts,
                    req,
                )
            }
        };
        let mut resp = Response::new(Body::empty());
        resp.headers_mut().insert(hyper::header::LOCATION, loc);
        *resp.status_mut() = match redirect_type {
            RedirectsKind::Permanent => StatusCode::MOVED_PERMANENTLY,
            RedirectsKind::Temporary => StatusCode::FOUND,
        };
        Some(Ok(resp))
    } else {
        // Handle internal rewrites
        *req.uri_mut() = match merge_uris(req.uri(), &dest) {
            Ok(uri) => uri,
            Err(err) => {
                return handle_error(
                    err.context("invalid rewrite target from current uri"),
                    opts,
                    req,
                )
            }
        };

        // Adjust Host header to allow rewriting to a different virtual host
        if let Some(host) = req.uri().host() {
            let mut host = host.to_owned();
            if let Some(port) = req.uri().port_u16() {
                host.push_str(&format!(":{}", port));
            }
            if let Ok(host) = host.parse() {
                req.headers_mut().insert(HOST, host);
            }
        }

        None
    }
}

fn merge_uris(orig_uri: &Uri, new_uri: &str) -> Result<Uri, Error> {
    let mut parts = new_uri.parse::<Uri>()?.into_parts();
    if parts.scheme.is_none() {
        parts.scheme = orig_uri.scheme().cloned();
    }
    if parts.authority.is_none() {
        parts.authority = orig_uri.authority().cloned();
    }
    if parts.path_and_query.is_none() {
        parts.path_and_query = orig_uri.path_and_query().cloned();
    }
    if let Some(path_and_query) = &mut parts.path_and_query {
        if let (None, Some(query)) = (path_and_query.query(), orig_uri.query()) {
            *path_and_query = [path_and_query.as_str(), "?", query]
                .into_iter()
                .collect::<String>()
                .parse()?;
        }
    }
    Ok(Uri::from_parts(parts)?)
}

/// It returns a rewrite's destination path if the current request uri
/// matches against the provided rewrites array.
pub fn rewrite_uri_path<'a>(
    uri_path: &'a str,
    rewrites_opts: Option<&'a [Rewrites]>,
) -> Option<&'a Rewrites> {
    if let Some(rewrites_vec) = rewrites_opts {
        for rewrites_entry in rewrites_vec {
            // Match source glob pattern against request uri path
            if rewrites_entry.source.is_match(uri_path) {
                return Some(rewrites_entry);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::pre_process;
    use crate::{
        handler::RequestHandlerOpts,
        settings::{file::RedirectsKind, Advanced, Rewrites},
        Error,
    };
    use hyper::{header::HOST, Body, Request, Response, StatusCode};
    use regex::Regex;

    fn make_request(host: &str, uri: &str) -> Request<Body> {
        let mut builder = Request::builder();
        if !host.is_empty() {
            builder = builder.header("Host", host);
        }
        builder.method("GET").uri(uri).body(Body::empty()).unwrap()
    }

    fn get_rewrites() -> Vec<Rewrites> {
        vec![
            Rewrites {
                source: Regex::new(r"/source1$").unwrap(),
                destination: "/destination1".into(),
                redirect: None,
            },
            Rewrites {
                source: Regex::new(r"/source2$").unwrap(),
                destination: "/destination2".into(),
                redirect: Some(RedirectsKind::Temporary),
            },
            Rewrites {
                source: Regex::new(r"/(prefix/)?(source3)/(.*)").unwrap(),
                destination: "/destination3/$2/$3".into(),
                redirect: Some(RedirectsKind::Permanent),
            },
            Rewrites {
                source: Regex::new(r"/(source4)/(.*)").unwrap(),
                destination: "http://example.net:1234/destination4/$1?$2".into(),
                redirect: None,
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
    fn test_no_rewrites() {
        let mut req = make_request("", "/");
        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: None,
                ..Default::default()
            },
            &mut req
        )
        .is_none());
        assert_eq!(req.uri(), "/");

        let mut req = make_request("", "/");
        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: Some(Advanced {
                    rewrites: None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            &mut req
        )
        .is_none());
        assert_eq!(req.uri(), "/");
    }

    #[test]
    fn test_no_match() {
        let mut req = make_request("example.com", "/source2/whatever");
        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: Some(Advanced {
                    rewrites: Some(get_rewrites()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &mut req
        )
        .is_none());
        assert_eq!(req.uri(), "/source2/whatever");
    }

    #[test]
    fn test_match() {
        let mut req = make_request("", "/source1?query");
        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: Some(Advanced {
                    rewrites: Some(get_rewrites()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &mut req
        )
        .is_none());
        assert_eq!(req.uri(), "/destination1?query");

        let mut req = make_request("", "/source2");
        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        rewrites: Some(get_rewrites()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &mut req
            )),
            Some((StatusCode::FOUND, "/destination2".into()))
        );

        let mut req = make_request("", "/source3/whatever");
        assert_eq!(
            is_redirect(pre_process(
                &RequestHandlerOpts {
                    advanced_opts: Some(Advanced {
                        rewrites: Some(get_rewrites()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &mut req
            )),
            Some((
                StatusCode::MOVED_PERMANENTLY,
                "/destination3/source3/whatever".into()
            ))
        );

        let mut req = make_request("example.com", "/source4/whatever?query");
        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: Some(Advanced {
                    rewrites: Some(get_rewrites()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &mut req
        )
        .is_none());
        assert_eq!(
            req.uri(),
            "http://example.net:1234/destination4/source4?whatever"
        );
        assert_eq!(
            req.headers()
                .get(HOST)
                .map(|h| h.to_str().unwrap())
                .unwrap_or(""),
            "example.net:1234"
        );
    }
}
