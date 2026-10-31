// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the health endpoints.
//!
//! SWS exposes the Kubernetes-style `/livez` (liveness) and `/readyz` (readiness) probes.
//! Both return `200 OK`, except `/readyz` returns `503 Service Unavailable` once a shutdown
//! signal is received, so load balancers stop routing to SWS during the grace period, or
//! while the root directory is not an accessible directory.
//!

use headers::{ContentType, HeaderMapExt};
use hyper::{Method, Request, Response, StatusCode};
use std::sync::atomic::Ordering;

use crate::body::Body;
use crate::{Error, handler::RequestHandlerOpts};

/// Path of the liveness probe endpoint.
const LIVENESS_PATH: &str = "/livez";

/// Path of the readiness probe endpoint.
const READINESS_PATH: &str = "/readyz";

/// Initializes the health endpoints (`/livez` and `/readyz`).
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.health = enabled;
    tracing::info!(
        enabled,
        "health endpoints ({LIVENESS_PATH}, {READINESS_PATH})"
    );
}

/// Answers `GET` and `HEAD` requests to `/livez` and `/readyz` with a `text/plain` body:
/// `200 OK`, or `503` for `/readyz` while shutting down or without a root directory.
/// Other requests fall through.
pub fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    if !opts.health || !is_health_endpoint(req) {
        return None;
    }

    let is_head = match *req.method() {
        Method::HEAD => true,
        Method::GET => false,
        _ => return None,
    };

    let (status, text) = match readiness_failure(opts, req) {
        Some(reason) => (StatusCode::SERVICE_UNAVAILABLE, reason),
        None => (StatusCode::OK, "OK"),
    };

    let body = if is_head {
        crate::body::empty()
    } else {
        crate::body::full(text)
    };
    let mut resp = Response::new(body);
    *resp.status_mut() = status;
    // SECURITY: The body is a literal ASCII string, so advertise it
    // as `text/plain` instead of `text/html`. This eliminates any chance
    // of a downstream proxy / reverse-CDN rendering this endpoint as
    // HTML and prevents a future contributor from accidentally
    // introducing markup into a probe response.
    resp.headers_mut().typed_insert(ContentType::text_utf8());
    Some(Ok(resp))
}

/// Returns why `/readyz` must fail, or `None` when ready or for `/livez`.
fn readiness_failure<T>(opts: &RequestHandlerOpts, req: &Request<T>) -> Option<&'static str> {
    if req.uri().path() != READINESS_PATH {
        return None;
    }
    if opts.shutting_down.load(Ordering::Relaxed) {
        return Some("shutting down");
    }
    // Same blocking `stat(2)` the file handler runs for every request.
    if !std::fs::metadata(&opts.root_dir).is_ok_and(|m| m.is_dir()) {
        return Some("root directory unavailable");
    }
    None
}

/// Reports whether the request path is one of the health endpoints, ignoring the query.
#[inline]
pub(crate) fn is_health_endpoint<T>(req: &Request<T>) -> bool {
    matches!(req.uri().path(), LIVENESS_PATH | READINESS_PATH)
}

#[cfg(test)]
mod tests {
    use super::{is_health_endpoint, pre_process};
    use crate::body::Body;
    use crate::handler::RequestHandlerOpts;
    use http_body_util::BodyExt;
    use hyper::{Request, StatusCode, header::CONTENT_TYPE};
    use std::sync::atomic::Ordering;

    const PROBES: [&str; 2] = ["/livez", "/readyz"];

    fn make_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(crate::body::empty())
            .unwrap()
    }

    fn enabled() -> RequestHandlerOpts {
        RequestHandlerOpts {
            health: true,
            root_dir: "tests/fixtures/public".into(),
            ..Default::default()
        }
    }

    #[test]
    fn health_disabled_skips_probes() {
        let opts = RequestHandlerOpts {
            health: false,
            ..Default::default()
        };
        for path in PROBES {
            assert!(
                pre_process(&opts, &make_request("GET", path)).is_none(),
                "{path}"
            );
        }
    }

    #[tokio::test]
    async fn health_get_probe_returns_ok_text() {
        for path in PROBES {
            let resp = pre_process(&enabled(), &make_request("GET", path))
                .expect("probe must be handled")
                .expect("probe must not fail");
            assert_eq!(resp.status(), StatusCode::OK, "{path}");
            assert_eq!(
                resp.headers()[CONTENT_TYPE],
                "text/plain; charset=utf-8",
                "{path}"
            );
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body.as_ref(), b"OK", "{path}");
        }
    }

    #[tokio::test]
    async fn health_head_probe_returns_empty_body() {
        for path in PROBES {
            let resp = pre_process(&enabled(), &make_request("HEAD", path))
                .expect("probe must be handled")
                .expect("probe must not fail");
            assert_eq!(resp.status(), StatusCode::OK, "{path}");
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            assert!(body.is_empty(), "{path}");
        }
    }

    #[tokio::test]
    async fn health_readyz_fails_while_shutting_down() {
        let opts = enabled();
        opts.shutting_down.store(true, Ordering::Relaxed);

        let resp = pre_process(&opts, &make_request("GET", "/readyz"))
            .expect("probe must be handled")
            .expect("probe must not fail");
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), b"shutting down");

        let resp = pre_process(&opts, &make_request("HEAD", "/readyz"))
            .expect("probe must be handled")
            .expect("probe must not fail");
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

        let resp = pre_process(&opts, &make_request("GET", "/livez"))
            .expect("probe must be handled")
            .expect("probe must not fail");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn health_probe_ignores_query() {
        for path in PROBES {
            let uri = format!("{path}?verbose");
            assert!(
                pre_process(&enabled(), &make_request("GET", &uri)).is_some(),
                "{uri}"
            );
        }
    }

    #[test]
    fn health_probe_other_methods_fall_through() {
        for path in PROBES {
            for method in ["POST", "PUT", "DELETE", "OPTIONS"] {
                assert!(
                    pre_process(&enabled(), &make_request(method, path)).is_none(),
                    "{method} {path}"
                );
            }
        }
    }

    #[test]
    fn health_legacy_and_similar_paths_fall_through() {
        for path in [
            "/health", "/healthz", "/livez/", "/readyz/", "/livez2", "/LIVEZ", "/a/livez",
        ] {
            let req = make_request("GET", path);
            assert!(!is_health_endpoint(&req), "{path}");
            assert!(pre_process(&enabled(), &req).is_none(), "{path}");
        }
    }

    #[tokio::test]
    async fn health_readyz_fails_without_root_dir() {
        for root in ["tests/fixtures/missing", "tests/fixtures/public/index.htm"] {
            let opts = RequestHandlerOpts {
                root_dir: root.into(),
                ..enabled()
            };

            let resp = pre_process(&opts, &make_request("GET", "/readyz"))
                .expect("probe must be handled")
                .expect("probe must not fail");
            assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE, "{root}");
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body.as_ref(), b"root directory unavailable", "{root}");

            let resp = pre_process(&opts, &make_request("GET", "/livez"))
                .expect("probe must be handled")
                .expect("probe must not fail");
            assert_eq!(resp.status(), StatusCode::OK, "{root}");
        }
    }
}
