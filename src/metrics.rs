// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the experimental metrics endpoint.
//!

use headers::{ContentType, HeaderMapExt};
use hyper::{Body, Request, Response};
use prometheus::{default_registry, Encoder, TextEncoder};

use crate::{handler::RequestHandlerOpts, http_ext::MethodExt, Error};

/// Initializes the metrics endpoint.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.experimental_metrics = enabled;
    server_info!("metrics endpoint (experimental): enabled={enabled}");

    if enabled {
        default_registry()
            .register(Box::new(
                tokio_metrics_collector::default_runtime_collector(),
            ))
            .unwrap();
    }
}

/// Handles metrics requests
pub fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    if !opts.experimental_metrics {
        return None;
    }

    let uri = req.uri();
    if uri.path() != "/metrics" {
        return None;
    }

    let method = req.method();
    if !method.is_get() && !method.is_head() {
        return None;
    }

    let body = if method.is_get() {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&default_registry().gather(), &mut buffer)
            .unwrap();
        let data = String::from_utf8(buffer).unwrap();
        Body::from(data)
    } else {
        Body::empty()
    };
    let mut resp = Response::new(body);
    resp.headers_mut()
        .typed_insert(ContentType::from(mime_guess::mime::TEXT_PLAIN_UTF_8));
    Some(Ok(resp))
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
    fn test_metrics_disabled() {
        assert!(pre_process(
            &RequestHandlerOpts {
                experimental_metrics: false,
                ..Default::default()
            },
            &make_request("GET", "/metrics")
        )
        .is_none());
    }

    #[test]
    fn test_wrong_uri() {
        assert!(pre_process(
            &RequestHandlerOpts {
                experimental_metrics: true,
                ..Default::default()
            },
            &make_request("GET", "/metrics2")
        )
        .is_none());
    }

    #[test]
    fn test_wrong_method() {
        assert!(pre_process(
            &RequestHandlerOpts {
                experimental_metrics: true,
                ..Default::default()
            },
            &make_request("POST", "/metrics")
        )
        .is_none());
    }

    #[test]
    fn test_correct_request() {
        assert!(pre_process(
            &RequestHandlerOpts {
                experimental_metrics: true,
                ..Default::default()
            },
            &make_request("GET", "/metrics")
        )
        .is_some());
    }
}
