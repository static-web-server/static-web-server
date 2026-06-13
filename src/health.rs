// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the health endpoint.
//!

use headers::{ContentType, HeaderMapExt};
use hyper::{Method, Request, Response};

use crate::body::Body;
use crate::{Error, handler::RequestHandlerOpts};

/// Initializes the health endpoint.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.health = enabled;
    tracing::info!(enabled, "health endpoint");
}

/// Handles health requests.
pub fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    if !opts.health {
        return None;
    }

    if !is_health_endpoint(req) {
        return None;
    }

    let body = match *req.method() {
        Method::HEAD => crate::body::empty(),
        Method::GET => crate::body::full("OK"),
        _ => return None,
    };

    let mut resp = Response::new(body);
    // SECURITY: The body is a literal `OK` ASCII string, so advertise it
    // as `text/plain` instead of `text/html`. This eliminates any chance
    // of a downstream proxy / reverse-CDN rendering this endpoint as
    // HTML and prevents a future contributor from accidentally
    // introducing markup into a probe response.
    resp.headers_mut().typed_insert(ContentType::text_utf8());
    Some(Ok(resp))
}

pub(crate) fn is_health_endpoint<T>(req: &Request<T>) -> bool {
    req.uri().path() == "/health"
}

#[cfg(test)]
mod tests {
    use super::pre_process;
    use crate::body::Body;
    use crate::handler::RequestHandlerOpts;
    use hyper::Request;

    fn make_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(crate::body::empty())
            .unwrap()
    }

    #[test]
    fn test_health_disabled() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    health: false,
                    ..Default::default()
                },
                &make_request("GET", "/health"),
            )
            .is_none()
        );
    }

    #[test]
    fn test_wrong_uri() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    health: true,
                    ..Default::default()
                },
                &make_request("GET", "/health2"),
            )
            .is_none()
        );
    }

    #[test]
    fn test_wrong_method() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    health: true,
                    ..Default::default()
                },
                &make_request("POST", "/health"),
            )
            .is_none()
        );
    }

    #[test]
    fn test_correct_request() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    health: true,
                    ..Default::default()
                },
                &make_request("GET", "/health"),
            )
            .is_some()
        );
    }
}
