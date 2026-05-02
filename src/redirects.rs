// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Redirection module to handle config redirect URLs with pattern matching support.
//!

use headers::HeaderValue;
use hyper::{Request, Response, StatusCode};
use regex_lite::Regex;

use crate::body::Body;
use crate::{Error, error_page, handler::RequestHandlerOpts, settings::Redirects};

/// Applies redirect rules to a request if necessary.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
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
        uri_host.push_str(&format!(":{uri_port}"));
    }
    let matched = get_redirection(&uri_host, uri_path, Some(redirects))?;
    let dest = match replace_placeholders(
        uri_path,
        &matched.source,
        &matched.destination,
        &matched.replacer,
    ) {
        Ok(dest) => dest,
        Err(err) => return handle_error(err, opts, req),
    };

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
pub(crate) fn replace_placeholders(
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
    }
}
