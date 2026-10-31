// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module providing the health endpoints.
//!
//! SWS exposes the Kubernetes-style `/livez` (liveness) and `/readyz` (readiness) probes.
//! SWS keeps no state that could make it alive but not ready, so both paths return the
//! same `200 OK` response.
//!

use headers::{ContentType, HeaderMapExt};
use hyper::{Method, Request, Response};

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

/// Answers `GET` and `HEAD` requests to `/livez` and `/readyz` with `200 OK` and a
/// `text/plain` `OK` body. Any other request falls through to the next pipeline step.
pub fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    if !opts.health || !is_health_endpoint(req) {
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
}
