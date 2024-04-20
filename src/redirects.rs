// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Redirection module to handle config redirect URLs with pattern matching support.
//!

use headers::HeaderValue;
use hyper::{Body, Request, Response, StatusCode};

use crate::{error_page, handler::RequestHandlerOpts, settings::Redirects, Error};

pub(crate) fn pre_process(
    opts: &RequestHandlerOpts,
    req: &Request<Body>,
) -> Option<Result<Response<Body>, Error>> {
    let redirects = opts.advanced_opts.as_ref()?.redirects.as_deref()?;

    let uri = req.uri();
    let uri_path = uri.path();
    let host = req
        .headers()
        .get(http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let mut uri_host = uri.host().unwrap_or(host).to_owned();
    if let Some(uri_port) = uri.port_u16() {
        uri_host.push_str(&format!(":{}", uri_port));
    }
    if let Some(matched) = get_redirection(&uri_host, uri_path, Some(redirects)) {
        // Redirects: Handle replacements (placeholders)
        let regex_caps = if let Some(regex_caps) = matched.source.captures(uri_path) {
            regex_caps
        } else {
            return handle_error(
                "unexpected regex failure",
                "extracting captures failed",
                opts,
                req,
            );
        };

        let caps_range = 0..regex_caps.len();
        let caps = caps_range
            .clone()
            .map(|i| regex_caps.get(i).map(|s| s.as_str()).unwrap_or(""))
            .collect::<Vec<&str>>();

        let patterns = caps_range
            .map(|i| format!("${}", i))
            .collect::<Vec<String>>();

        let dest = &matched.destination;

        tracing::debug!("url redirects glob pattern: {:?}", patterns);
        tracing::debug!("url redirects regex equivalent: {}", matched.source);
        tracing::debug!("url redirects glob pattern captures: {:?}", caps);
        tracing::debug!("url redirects glob pattern destination: {:?}", dest);

        let ac = match aho_corasick::AhoCorasick::new(patterns) {
            Ok(ac) => ac,
            Err(err) => {
                return handle_error("failed creating Aho-Corasick matcher", err, opts, req)
            }
        };
        let dest = match ac.try_replace_all(dest, &caps) {
            Ok(dest) => dest.to_string(),
            Err(err) => return handle_error("failed replacing captures", err, opts, req),
        };

        tracing::debug!(
            "url redirects glob pattern destination replaced: {:?}",
            dest
        );
        match HeaderValue::from_str(&dest) {
            Ok(loc) => {
                let mut resp = Response::new(Body::empty());
                resp.headers_mut().insert(hyper::header::LOCATION, loc);
                *resp.status_mut() = matched.kind;
                tracing::trace!(
                    "uri matches redirects glob pattern, redirecting with status '{}'",
                    matched.kind
                );
                Some(Ok(resp))
            }
            Err(err) => handle_error("invalid header value from current uri", err, opts, req),
        }
    } else {
        None
    }
}

fn handle_error<E: std::fmt::Display>(
    msg: &str,
    err: E,
    opts: &RequestHandlerOpts,
    req: &Request<Body>,
) -> Option<Result<Response<Body>, Error>> {
    tracing::error!("{msg}: {err}");
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
    use crate::{
        handler::RequestHandlerOpts,
        settings::{Advanced, Redirects},
        Error,
    };
    use hyper::{Body, Request, Response, StatusCode};
    use regex::Regex;

    fn make_request(host: &str, uri: &str) -> Request<Body> {
        let mut builder = Request::builder();
        if !host.is_empty() {
            builder = builder.header("Host", host);
        }
        builder.method("GET").uri(uri).body(Body::empty()).unwrap()
    }

    fn get_redirects() -> Vec<Redirects> {
        vec![
            Redirects {
                host: None,
                source: Regex::new(r"/source1$").unwrap(),
                destination: "/destination1".into(),
                kind: StatusCode::FOUND,
            },
            Redirects {
                host: Some("example.com".into()),
                source: Regex::new(r"/source2$").unwrap(),
                destination: "/destination2".into(),
                kind: StatusCode::MOVED_PERMANENTLY,
            },
            Redirects {
                host: Some("example.info".into()),
                source: Regex::new(r"/(prefix/)?(source3)/(.*)").unwrap(),
                destination: "/destination3/$2/$3".into(),
                kind: StatusCode::MOVED_PERMANENTLY,
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
        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: None,
                ..Default::default()
            },
            &make_request("", "/")
        )
        .is_none());

        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: Some(Advanced {
                    redirects: None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            &make_request("", "/")
        )
        .is_none());
    }

    #[test]
    fn test_no_match() {
        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: Some(Advanced {
                    redirects: Some(get_redirects()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &make_request("example.com", "/source2/whatever")
        )
        .is_none());

        assert!(pre_process(
            &RequestHandlerOpts {
                advanced_opts: Some(Advanced {
                    redirects: Some(get_redirects()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &make_request("", "/source2")
        )
        .is_none());
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
    }
}
