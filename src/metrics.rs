// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the metrics endpoint and HTTP-level instrumentation.
//!

use std::sync::LazyLock;

use headers::{ContentType, HeaderMapExt};
use hyper::{Request, Response, StatusCode};
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, TextEncoder,
    default_registry,
};

use crate::body::Body;
use crate::{Error, exts::http::MethodExt, handler::RequestHandlerOpts, virtual_hosts};

/// Label for a request method or host outside the known set
const OTHER_LABEL: &str = "other";

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
/// Tokio runtime metrics are additionally registered when the `experimental`
/// feature is enabled and built with `RUSTFLAGS="--cfg tokio_unstable"`.
///
/// # Security
///
/// The `/metrics` endpoint is exposed **without any built-in access
/// control** \u2014 it is intentionally unauthenticated so that a sidecar
/// Prometheus scraper can reach it cheaply. Operators MUST place SWS
/// behind a reverse proxy or network policy that restricts `/metrics`
/// to trusted scrapers; otherwise an unauthenticated client could
/// enumerate vhost names, request volumes, and latency distributions
/// (information disclosure).
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.metrics_enabled = enabled;
    tracing::info!(enabled, "metrics endpoint");
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
            tracing::info!(enabled = true, "tokio runtime metrics");
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
        crate::body::full(data)
    } else {
        crate::body::empty()
    };
    let mut resp = Response::new(body);
    resp.headers_mut()
        .typed_insert(ContentType::from(mime_guess::mime::TEXT_PLAIN_UTF_8));
    Some(Ok(resp))
}

/// Records HTTP request metrics after a response is produced.
///
/// Labels taken from the request come from a fixed set, so clients can't
/// create new series: `method` is `GET`, `HEAD`, `OPTIONS` or `other`, and
/// `host` is a name from `[advanced.virtual-hosts]` or `other`.
pub fn record_request<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    status: StatusCode,
    bytes: u64,
    elapsed: f64,
) {
    if req.uri().path() == "/metrics" {
        return;
    }
    let m = method_label(req);
    let host = host_label(opts, req);
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

fn method_label<T>(req: &Request<T>) -> &str {
    let method = req.method();
    if method.is_allowed() {
        method.as_str()
    } else {
        OTHER_LABEL
    }
}

fn host_label<'a, T>(opts: &'a RequestHandlerOpts, req: &Request<T>) -> &'a str {
    let vhosts = opts
        .advanced_opts
        .as_ref()
        .and_then(|advanced| advanced.virtual_hosts.as_deref());
    let (Some(vhosts), Some(host)) = (vhosts, virtual_hosts::request_host(req)) else {
        return OTHER_LABEL;
    };
    vhosts
        .iter()
        .find(|vhost| vhost.host == host)
        .map_or(OTHER_LABEL, |vhost| vhost.host.as_str())
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
    use crate::settings::{Advanced, VirtualHosts};
    use hyper::Request;
    use prometheus::core::Collector;
    use std::path::PathBuf;
    use std::sync::Mutex;

    fn make_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(crate::body::empty())
            .unwrap()
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

    /// The counters are global, so tests that record requests run one at a time
    static RECORD_LOCK: Mutex<()> = Mutex::new(());

    fn vhost_opts(hosts: &[&str]) -> RequestHandlerOpts {
        let virtual_hosts = hosts
            .iter()
            .map(|host| VirtualHosts {
                host: host.to_string(),
                root: PathBuf::from("/var/www"),
            })
            .collect();
        RequestHandlerOpts {
            advanced_opts: Some(Advanced {
                virtual_hosts: Some(virtual_hosts),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn host_request(method: &str, host: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri("/index.html")
            .header(hyper::header::HOST, host)
            .body(crate::body::empty())
            .unwrap()
    }

    fn requests_total(labels: [&str; 3]) -> u64 {
        HTTP_REQUESTS_TOTAL.with_label_values(&labels).get()
    }

    fn label_values(name: &str) -> Vec<String> {
        HTTP_REQUESTS_TOTAL
            .collect()
            .iter()
            .flat_map(|family| family.get_metric())
            .flat_map(|metric| metric.get_label())
            .filter(|label| label.name() == name)
            .map(|label| label.value().to_owned())
            .collect()
    }

    #[test]
    fn test_record_request_labels_virtual_host() {
        let _lock = RECORD_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let opts = vhost_opts(&["example.com"]);
        let labels = ["GET", "2xx", "example.com"];
        let before = requests_total(labels);
        let bytes_before = HTTP_RESPONSE_BYTES_TOTAL.with_label_values(&labels).get();

        let req = host_request("GET", "example.com:8080");
        record_request(&opts, &req, StatusCode::OK, 1024, 0.005);

        assert_eq!(requests_total(labels), before + 1);
        assert_eq!(
            HTTP_RESPONSE_BYTES_TOTAL.with_label_values(&labels).get(),
            bytes_before + 1024
        );
    }

    #[test]
    fn test_record_request_labels_unknown_host_as_other() {
        let _lock = RECORD_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let labels = ["GET", "2xx", "other"];
        let before = requests_total(labels);

        // With and without virtual hosts configured
        let cases = [
            (vhost_opts(&["example.com"]), "unknown-1.test"),
            (RequestHandlerOpts::default(), "unknown-2.test"),
        ];
        for (opts, host) in &cases {
            let req = host_request("GET", host);
            record_request(opts, &req, StatusCode::OK, 0, 0.001);
        }

        assert_eq!(requests_total(labels), before + 2);
        let hosts = label_values("host");
        assert!(!hosts.iter().any(|h| h.starts_with("unknown-")));
    }

    #[test]
    fn test_record_request_labels_unknown_method_as_other() {
        let _lock = RECORD_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let labels = ["other", "4xx", "other"];
        let before = requests_total(labels);

        let req = host_request("FOO", "example.com");
        let opts = RequestHandlerOpts::default();
        record_request(&opts, &req, StatusCode::METHOD_NOT_ALLOWED, 0, 0.001);

        assert_eq!(requests_total(labels), before + 1);
        assert!(!label_values("method").contains(&"FOO".to_owned()));
    }

    #[test]
    fn test_record_request_skips_metrics_path() {
        let _lock = RECORD_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let labels = ["GET", "2xx", "other"];
        let before = requests_total(labels);

        let req = make_request("GET", "/metrics");
        let opts = RequestHandlerOpts::default();
        record_request(&opts, &req, StatusCode::OK, 0, 0.001);

        assert_eq!(requests_total(labels), before);
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
