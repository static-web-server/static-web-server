// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the health endpoint.
//!

use headers::{ContentType, HeaderMapExt};
use hyper::{Body, Request, Response};

use crate::{handler::RequestHandlerOpts, http_ext::MethodExt, server_info, Error};

/// Encapsulates functionality required for the health handler.
pub struct HealthHandler {}

impl HealthHandler {
    /// Initializes the health endpoint.
    pub fn init(enabled: bool) {
        server_info!("health endpoint: enabled={enabled}");
    }

    /// Handles health requests
    pub fn pre_process(
        opts: &RequestHandlerOpts,
        req: &Request<Body>,
        remote_addr_str: &str,
    ) -> Option<Result<Response<Body>, Error>> {
        if !opts.health {
            return None;
        }

        let uri = req.uri();
        if uri.path() != "/health" {
            return None;
        }

        let method = req.method();
        if !method.is_get() && !method.is_head() {
            return None;
        }

        tracing::debug!(
            "incoming request: method={} uri={}{}",
            method,
            uri,
            remote_addr_str,
        );

        let body = if method.is_get() {
            Body::from("OK")
        } else {
            Body::empty()
        };

        let mut resp = Response::new(body);
        resp.headers_mut().typed_insert(ContentType::html());
        Some(Ok(resp))
    }
}

#[cfg(test)]
mod tests {
    use super::HealthHandler;
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
        assert!(HealthHandler::pre_process(
            &RequestHandlerOpts {
                health: false,
                ..Default::default()
            },
            &make_request("GET", "/health"),
            ""
        )
        .is_none());
    }

    #[test]
    fn test_wrong_uri() {
        assert!(HealthHandler::pre_process(
            &RequestHandlerOpts {
                health: true,
                ..Default::default()
            },
            &make_request("GET", "/health2"),
            ""
        )
        .is_none());
    }

    #[test]
    fn test_wrong_method() {
        assert!(HealthHandler::pre_process(
            &RequestHandlerOpts {
                health: true,
                ..Default::default()
            },
            &make_request("POST", "/health"),
            ""
        )
        .is_none());
    }

    #[test]
    fn test_correct_request() {
        assert!(HealthHandler::pre_process(
            &RequestHandlerOpts {
                health: true,
                ..Default::default()
            },
            &make_request("GET", "/health"),
            ""
        )
        .is_some());
    }
}
