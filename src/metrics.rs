// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the experimental metrics endpoint and HTTP-level instrumentation.
//!

use std::sync::LazyLock;

use headers::{ContentType, HeaderMapExt};
use hyper::{Body, Request, Response, StatusCode};
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, TextEncoder,
    default_registry,
};

use crate::{Error, handler::RequestHandlerOpts, http_ext::MethodExt};

// Histogram buckets tuned for static file serving (50µs to 10s).
// Sub-millisecond range captures cache hits and small in-memory responses.
const LATENCY_BUCKETS: &[f64] = &[
    0.00005, 0.0001, 0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,
    2.5, 5.0, 10.0,
];

static HTTP_REQUESTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    IntCounterVec::new(
        Opts::new(
            "sws_http_requests_total",
            "Total HTTP requests by method, status class, and host.",
        ),
        &["method", "status", "host"],
    )
    .unwrap()
});

static HTTP_REQUEST_DURATION_SECONDS: LazyLock<HistogramVec> = LazyLock::new(|| {
    HistogramVec::new(
        HistogramOpts::new(
            "sws_http_request_duration_seconds",
            "HTTP request duration in seconds by method, status class, and host.",
        )
        .buckets(LATENCY_BUCKETS.to_vec()),
        &["method", "status", "host"],
    )
    .unwrap()
});

static HTTP_RESPONSE_BYTES_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    IntCounterVec::new(
        Opts::new(
            "sws_http_response_bytes_total",
            "Total HTTP response bytes (Content-Length) by method, status class, and host.",
        ),
        &["method", "status", "host"],
    )
    .unwrap()
});

static HTTP_REQUESTS_INFLIGHT: LazyLock<IntGauge> = LazyLock::new(|| {
    IntGauge::new(
        "sws_http_requests_inflight",
        "Number of HTTP requests currently being processed.",
    )
    .unwrap()
});

static HTTP_CONNECTIONS_ACTIVE: LazyLock<IntGauge> = LazyLock::new(|| {
    IntGauge::new(
        "sws_http_connections_active",
        "Number of currently active HTTP connections.",
    )
    .unwrap()
});

/// Initializes the metrics endpoint and registers HTTP-level collectors.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.experimental_metrics = enabled;
    tracing::info!("metrics endpoint (experimental): enabled={enabled}");

    if enabled {
        let registry = default_registry();
        registry
            .register(Box::new(
                tokio_metrics_collector::default_runtime_collector(),
            ))
            .unwrap();
        registry
            .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
            .unwrap();
        registry
            .register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone()))
            .unwrap();
        registry
            .register(Box::new(HTTP_RESPONSE_BYTES_TOTAL.clone()))
            .unwrap();
        registry
            .register(Box::new(HTTP_REQUESTS_INFLIGHT.clone()))
            .unwrap();
        registry
            .register(Box::new(HTTP_CONNECTIONS_ACTIVE.clone()))
            .unwrap();
    }
}

/// Handles metrics requests.
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

/// Records HTTP request metrics after a response is produced.
pub fn record_request<T>(req: &Request<T>, status: StatusCode, bytes: u64, elapsed: f64) {
    if req.uri().path() == "/metrics" {
        return;
    }
    let m = req.method().as_str();
    let host = req
        .headers()
        .get(hyper::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let sc = status_class(status.as_u16());
    HTTP_REQUESTS_TOTAL.with_label_values(&[m, sc, host]).inc();
    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[m, sc, host])
        .observe(elapsed);
    if bytes > 0 {
        HTTP_RESPONSE_BYTES_TOTAL
            .with_label_values(&[m, sc, host])
            .inc_by(bytes);
    }
}

/// Increments the inflight requests gauge.
pub fn inc_requests_inflight() {
    HTTP_REQUESTS_INFLIGHT.inc();
}

/// Decrements the inflight requests gauge.
pub fn dec_requests_inflight() {
    HTTP_REQUESTS_INFLIGHT.dec();
}

/// Increments the active connections gauge.
pub fn inc_connections() {
    HTTP_CONNECTIONS_ACTIVE.inc();
}

/// Decrements the active connections gauge.
pub fn dec_connections() {
    HTTP_CONNECTIONS_ACTIVE.dec();
}

fn status_class(code: u16) -> &'static str {
    match code / 100 {
        1 => "1xx",
        2 => "2xx",
        3 => "3xx",
        4 => "4xx",
        _ => "5xx",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    experimental_metrics: false,
                    ..Default::default()
                },
                &make_request("GET", "/metrics")
            )
            .is_none()
        );
    }

    #[test]
    fn test_wrong_uri() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    experimental_metrics: true,
                    ..Default::default()
                },
                &make_request("GET", "/metrics2")
            )
            .is_none()
        );
    }

    #[test]
    fn test_wrong_method() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    experimental_metrics: true,
                    ..Default::default()
                },
                &make_request("POST", "/metrics")
            )
            .is_none()
        );
    }

    #[test]
    fn test_correct_request() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    experimental_metrics: true,
                    ..Default::default()
                },
                &make_request("GET", "/metrics")
            )
            .is_some()
        );
    }

    #[test]
    fn test_status_class() {
        assert_eq!(status_class(100), "1xx");
        assert_eq!(status_class(200), "2xx");
        assert_eq!(status_class(301), "3xx");
        assert_eq!(status_class(404), "4xx");
        assert_eq!(status_class(500), "5xx");
        assert_eq!(status_class(999), "5xx");
    }

    #[test]
    fn test_record_request() {
        let before = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "2xx", "example.com"])
            .get();
        let bytes_before = HTTP_RESPONSE_BYTES_TOTAL
            .with_label_values(&["GET", "2xx", "example.com"])
            .get();

        let req = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "example.com")
            .body(Body::empty())
            .unwrap();
        record_request(&req, StatusCode::OK, 1024, 0.005);

        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&["GET", "2xx", "example.com"])
                .get(),
            before + 1
        );
        assert_eq!(
            HTTP_RESPONSE_BYTES_TOTAL
                .with_label_values(&["GET", "2xx", "example.com"])
                .get(),
            bytes_before + 1024
        );
    }

    #[test]
    fn test_record_request_skips_metrics_path() {
        let before = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "2xx", ""])
            .get();

        let req = make_request("GET", "/metrics");
        record_request(&req, StatusCode::OK, 0, 0.001);

        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&["GET", "2xx", ""])
                .get(),
            before
        );
    }

    #[test]
    fn test_connection_gauge() {
        let before = HTTP_CONNECTIONS_ACTIVE.get();
        inc_connections();
        assert_eq!(HTTP_CONNECTIONS_ACTIVE.get(), before + 1);
        dec_connections();
        assert_eq!(HTTP_CONNECTIONS_ACTIVE.get(), before);
    }

    #[test]
    fn test_inflight_gauge() {
        let before = HTTP_REQUESTS_INFLIGHT.get();
        inc_requests_inflight();
        assert_eq!(HTTP_REQUESTS_INFLIGHT.get(), before + 1);
        dec_requests_inflight();
        assert_eq!(HTTP_REQUESTS_INFLIGHT.get(), before);
    }
}
