// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the metrics endpoint and HTTP-level instrumentation.
//!

use std::sync::LazyLock;

use headers::{ContentType, HeaderMapExt};
use hyper::{Body, Request, Response, StatusCode};
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, TextEncoder,
    default_registry,
};

use crate::settings::VirtualHosts;
use crate::{Error, handler::RequestHandlerOpts, http_ext::MethodExt};

// Histogram buckets tuned for static file serving (50µs to 10s).
// Sub-millisecond range captures cache hits and small in-memory responses.
const LATENCY_BUCKETS: &[f64] = &[
    0.00005, 0.0001, 0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,
    2.5, 5.0, 10.0,
];

/// Label used when a request label is not in an allowlist (unknown host or
/// unsupported method). Keeps Prometheus cardinality bounded (CWE-770).
const OTHER_LABEL: &str = "other";

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
/// Tokio runtime metrics are additionally registered when the `experimental`
/// feature is enabled and built with `RUSTFLAGS="--cfg tokio_unstable"`.
///
/// # Security
///
/// The `/metrics` endpoint is exposed **without any built-in access
/// control** — it is intentionally unauthenticated so that a sidecar
/// Prometheus scraper can reach it cheaply. Operators MUST place SWS
/// behind a reverse proxy or network policy that restricts `/metrics`
/// to trusted scrapers; otherwise an unauthenticated client could
/// enumerate vhost names, request volumes, and latency distributions
/// (information disclosure).
///
/// HTTP metric labels taken from the request are allowlisted. The `host`
/// label is only a name listed under `[advanced.virtual-hosts]`; anything
/// else is `other`. The `method` label is only GET, HEAD, or OPTIONS
/// (the methods SWS serves); anything else is `other`. This bounds series
/// cardinality when `--metrics=true`.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.metrics_enabled = enabled;
    tracing::info!("metrics endpoint: enabled={enabled}");
    if enabled {
        tracing::warn!(
            "metrics endpoint `/metrics` is unauthenticated; restrict access via reverse proxy or network policy"
        );
    }

    if enabled {
        let registry = default_registry();

        // Tokio runtime metrics (experimental, unix-only, requires tokio_unstable)
        #[cfg(all(unix, feature = "experimental"))]
        {
            if let Err(err) = registry.register(Box::new(
                tokio_metrics_collector::default_runtime_collector(),
            )) {
                tracing::debug!("tokio runtime metrics collector registration skipped: {err:?}");
            }
            tracing::info!("tokio runtime metrics: enabled");
        }

        // HTTP-level metrics
        if let Err(err) = registry.register(Box::new(HTTP_REQUESTS_TOTAL.clone())) {
            tracing::debug!("metrics collector registration skipped: {err:?}");
        }
        if let Err(err) = registry.register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone())) {
            tracing::debug!("metrics collector registration skipped: {err:?}");
        }
        if let Err(err) = registry.register(Box::new(HTTP_RESPONSE_BYTES_TOTAL.clone())) {
            tracing::debug!("metrics collector registration skipped: {err:?}");
        }
        if let Err(err) = registry.register(Box::new(HTTP_REQUESTS_INFLIGHT.clone())) {
            tracing::debug!("metrics collector registration skipped: {err:?}");
        }
        if let Err(err) = registry.register(Box::new(HTTP_CONNECTIONS_ACTIVE.clone())) {
            tracing::debug!("metrics collector registration skipped: {err:?}");
        }
    }
}

/// Handles metrics requests.
pub fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    if !opts.metrics_enabled {
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
        if let Err(err) = encoder.encode(&default_registry().gather(), &mut buffer) {
            return Some(Err(
                Error::new(err).context("failed to encode metrics output")
            ));
        }
        let data = match String::from_utf8(buffer) {
            Ok(data) => data,
            Err(err) => {
                return Some(Err(
                    Error::new(err).context("metrics output was not valid UTF-8")
                ));
            }
        };
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
///
/// `virtual_hosts` is the configured name-based vhost list (same source
/// as request routing). Unlisted or missing hosts collapse to `other`.
pub fn record_request<T>(
    req: &Request<T>,
    status: StatusCode,
    bytes: u64,
    elapsed: f64,
    virtual_hosts: Option<&[VirtualHosts]>,
) {
    if req.uri().path() == "/metrics" {
        return;
    }
    let m = method_label(req);
    let host = host_label(req, virtual_hosts);
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

/// Method used as the Prometheus `method` label: GET, HEAD, or OPTIONS,
/// or `other`. Custom/unsupported methods are collapsed so 405 responses
/// cannot mint unbounded series.
fn method_label<T>(req: &Request<T>) -> &str {
    let method = req.method();
    if method.is_allowed() {
        method.as_str()
    } else {
        OTHER_LABEL
    }
}

/// Host used as the Prometheus `host` label: a configured virtual host,
/// or `other`. Matches `virtual_hosts::get_real_root` host extraction
/// (HTTP/2 `:authority`, otherwise `Host` with a trailing port stripped).
fn host_label<'a, T>(req: &Request<T>, virtual_hosts: Option<&'a [VirtualHosts]>) -> &'a str {
    let Some(vhosts) = virtual_hosts.filter(|v| !v.is_empty()) else {
        return OTHER_LABEL;
    };
    let Some(host) = crate::virtual_hosts::request_host(req) else {
        return OTHER_LABEL;
    };
    vhosts
        .iter()
        .find(|v| v.host == host)
        .map(|v| v.host.as_str())
        .unwrap_or(OTHER_LABEL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::RequestHandlerOpts;
    use hyper::{Body, Request};
    use std::path::PathBuf;
    use std::sync::Mutex;

    static METRICS_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn make_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    fn make_vhost(host: &str) -> VirtualHosts {
        VirtualHosts {
            host: host.to_string(),
            root: PathBuf::from("/"),
        }
    }

    #[test]
    fn test_metrics_disabled() {
        assert!(
            pre_process(
                &RequestHandlerOpts {
                    metrics_enabled: false,
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
                    metrics_enabled: true,
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
                    metrics_enabled: true,
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
                    metrics_enabled: true,
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
    fn test_host_label_collapses_when_no_vhosts() {
        let req = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "poc-000001.invalid")
            .body(Body::empty())
            .unwrap();
        assert_eq!(host_label(&req, None), OTHER_LABEL);
        assert_eq!(host_label(&req, Some(&[])), OTHER_LABEL);
    }

    #[test]
    fn test_host_label_allows_configured_vhost() {
        let vhosts = [make_vhost("example.com")];
        let allowed = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "example.com")
            .body(Body::empty())
            .unwrap();
        let allowed_port = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "example.com:8080")
            .body(Body::empty())
            .unwrap();
        let unknown = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "poc-000002.invalid")
            .body(Body::empty())
            .unwrap();
        assert_eq!(host_label(&allowed, Some(&vhosts)), "example.com");
        assert_eq!(host_label(&allowed_port, Some(&vhosts)), "example.com");
        assert_eq!(host_label(&unknown, Some(&vhosts)), OTHER_LABEL);
    }

    #[test]
    fn test_record_request() {
        let _guard = METRICS_TEST_LOCK.lock().unwrap();
        let before = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "2xx", OTHER_LABEL])
            .get();
        let bytes_before = HTTP_RESPONSE_BYTES_TOTAL
            .with_label_values(&["GET", "2xx", OTHER_LABEL])
            .get();

        let req = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "example.com")
            .body(Body::empty())
            .unwrap();
        record_request(&req, StatusCode::OK, 1024, 0.005, None);

        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&["GET", "2xx", OTHER_LABEL])
                .get(),
            before + 1
        );
        assert_eq!(
            HTTP_RESPONSE_BYTES_TOTAL
                .with_label_values(&["GET", "2xx", OTHER_LABEL])
                .get(),
            bytes_before + 1024
        );
    }

    #[test]
    fn test_record_request_unknown_hosts_share_other_label() {
        let _guard = METRICS_TEST_LOCK.lock().unwrap();
        let before = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "2xx", OTHER_LABEL])
            .get();

        for host in ["poc-a.invalid", "poc-b.invalid", "poc-c.invalid"] {
            let req = Request::builder()
                .method("GET")
                .uri("/index.html")
                .header(hyper::header::HOST, host)
                .body(Body::empty())
                .unwrap();
            record_request(&req, StatusCode::OK, 0, 0.001, None);
        }

        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&["GET", "2xx", OTHER_LABEL])
                .get(),
            before + 3
        );
    }

    #[test]
    fn test_record_request_keeps_allowed_vhost_label() {
        let _guard = METRICS_TEST_LOCK.lock().unwrap();
        let vhosts = [make_vhost("example.com")];
        let before_ex = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "2xx", "example.com"])
            .get();
        let before_other = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "2xx", OTHER_LABEL])
            .get();

        let allowed = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "example.com")
            .body(Body::empty())
            .unwrap();
        record_request(&allowed, StatusCode::OK, 0, 0.001, Some(&vhosts));

        let unknown = Request::builder()
            .method("GET")
            .uri("/index.html")
            .header(hyper::header::HOST, "poc-d.invalid")
            .body(Body::empty())
            .unwrap();
        record_request(&unknown, StatusCode::OK, 0, 0.001, Some(&vhosts));

        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&["GET", "2xx", "example.com"])
                .get(),
            before_ex + 1
        );
        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&["GET", "2xx", OTHER_LABEL])
                .get(),
            before_other + 1
        );
    }

    #[test]
    fn test_record_request_skips_metrics_path() {
        let _guard = METRICS_TEST_LOCK.lock().unwrap();
        let before = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "2xx", OTHER_LABEL])
            .get();

        let req = make_request("GET", "/metrics");
        record_request(&req, StatusCode::OK, 0, 0.001, None);

        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&["GET", "2xx", OTHER_LABEL])
                .get(),
            before
        );
    }

    #[test]
    fn test_method_label_allows_supported_methods() {
        for method in ["GET", "HEAD", "OPTIONS"] {
            let req = make_request(method, "/index.html");
            assert_eq!(method_label(&req), method);
        }
    }

    #[test]
    fn test_method_label_collapses_unsupported_methods() {
        for method in ["POST", "PUT", "PATCH", "DELETE", "FOO"] {
            let req = make_request(method, "/index.html");
            assert_eq!(method_label(&req), OTHER_LABEL);
        }
    }

    #[test]
    fn test_record_request_unknown_methods_share_other_label() {
        let before = HTTP_REQUESTS_TOTAL
            .with_label_values(&[OTHER_LABEL, "4xx", OTHER_LABEL])
            .get();

        for method in ["POST", "FOO", "BAR"] {
            let req = make_request(method, "/index.html");
            record_request(&req, StatusCode::METHOD_NOT_ALLOWED, 0, 0.001, None);
        }

        assert_eq!(
            HTTP_REQUESTS_TOTAL
                .with_label_values(&[OTHER_LABEL, "4xx", OTHER_LABEL])
                .get(),
            before + 3
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
