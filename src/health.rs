// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the health endpoint.
//!

use headers::{ContentType, HeaderMapExt};
use hyper::{Body, Method, Request, Response};

use crate::{handler::RequestHandlerOpts, Error};

/// Initializes the health endpoint.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.health = enabled;
    server_info!("health endpoint: enabled={enabled}");
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
        Method::HEAD => Body::empty(),
        Method::GET => Body::from("OK"),
        _ => return None,
    };

    let mut resp = Response::new(body);
    resp.headers_mut().typed_insert(ContentType::html());
    Some(Ok(resp))
}

pub(crate) fn is_health_endpoint<T>(req: &Request<T>) -> bool {
    req.uri().path() == "/health"
}

#[cfg(test)]
mod tests {
    use super::pre_process;
    use crate::handler::RequestHandlerOpts;
    use hyper::{Body, Request};

    fn make_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn test_health_disabled() {
        assert!(pre_process(
            &RequestHandlerOpts {
                health: false,
                ..Default::default()
            },
            &make_request("GET", "/health"),
        )
        .is_none());
    }

    #[test]
    fn test_wrong_uri() {
        assert!(pre_process(
            &RequestHandlerOpts {
                health: true,
                ..Default::default()
            },
            &make_request("GET", "/health2"),
        )
        .is_none());
    }

    #[test]
    fn test_wrong_method() {
        assert!(pre_process(
            &RequestHandlerOpts {
                health: true,
                ..Default::default()
            },
            &make_request("POST", "/health"),
        )
        .is_none());
    }

    #[test]
    fn test_correct_request() {
        assert!(pre_process(
            &RequestHandlerOpts {
                health: true,
                ..Default::default()
            },
            &make_request("GET", "/health"),
        )
        .is_some());
    }
}
