// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to redirect HTTP requests to HTTPS.
//!

use headers::{HeaderMapExt, Host};
use hyper::{
    Request, Response, StatusCode,
    header::{HeaderValue, LOCATION},
};
use std::sync::Arc;

use crate::Result;
use crate::body::Body;

/// HTTPS redirect options.
pub struct RedirectOpts {
    /// HTTPS hostname to redirect to.
    pub https_hostname: String,
    /// HTTPS hostname port to redirect to.
    pub https_port: u16,
    /// Hostnames or IPS to redirect from.
    pub allowed_hosts: Vec<String>,
}

/// It redirects all requests from HTTP to HTTPS.
pub fn redirect_to_https<T>(
    req: &Request<T>,
    opts: Arc<RedirectOpts>,
) -> Result<Response<Body>, StatusCode> {
    if let Some(ref host) = req.headers().typed_get::<Host>() {
        let from_hostname = host.hostname();
        if !opts
            .allowed_hosts
            .iter()
            .any(|s| s.as_str() == from_hostname)
        {
            tracing::debug!("redirect host is not allowed!");
            return Err(StatusCode::BAD_REQUEST);
        }

        let url = format!(
            "https://{}:{}{}",
            opts.https_hostname,
            opts.https_port,
            req.uri()
        );
        tracing::debug!("https redirect to {}", url);

        let location = match HeaderValue::from_str(&url) {
            Ok(location) => location,
            Err(err) => {
                tracing::error!("invalid https redirect location `{url}`: {err:?}");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
        resp.headers_mut().insert(LOCATION, location);
        return Ok(resp);
    }

    tracing::debug!("redirect host was not determined!");
    Err(StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod tests {
    use hyper::{Method, Request, StatusCode, header::LOCATION};
    use std::sync::Arc;

    use super::{RedirectOpts, redirect_to_https};

    fn make_opts(hostname: &str, port: u16, allowed: &[&str]) -> Arc<RedirectOpts> {
        Arc::new(RedirectOpts {
            https_hostname: hostname.to_owned(),
            https_port: port,
            allowed_hosts: allowed.iter().map(|s| s.to_string()).collect(),
        })
    }

    fn request_with_host(host: &str, path: &str) -> Request<()> {
        Request::builder()
            .method(Method::GET)
            .uri(path)
            .header("host", host)
            .body(())
            .unwrap()
    }

    #[test]
    fn redirects_allowed_host_to_https() {
        let req = request_with_host("example.com", "/foo/bar");
        let opts = make_opts("example.com", 443, &["example.com"]);
        let resp = redirect_to_https(&req, opts).unwrap();
        assert_eq!(resp.status(), StatusCode::MOVED_PERMANENTLY);
        let location = resp.headers().get(LOCATION).unwrap().to_str().unwrap();
        assert_eq!(location, "https://example.com:443/foo/bar");
    }

    #[test]
    fn rejects_disallowed_host() {
        let req = request_with_host("attacker.com", "/");
        let opts = make_opts("example.com", 443, &["example.com"]);
        let err = redirect_to_https(&req, opts).unwrap_err();
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_request_without_host_header() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/foo")
            .body(())
            .unwrap();
        let opts = make_opts("example.com", 443, &["example.com"]);
        let err = redirect_to_https(&req, opts).unwrap_err();
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn includes_path_and_query_in_redirect() {
        let req = request_with_host("example.com", "/page?key=val");
        let opts = make_opts("example.com", 8443, &["example.com"]);
        let resp = redirect_to_https(&req, opts).unwrap();
        let location = resp.headers().get(LOCATION).unwrap().to_str().unwrap();
        assert_eq!(location, "https://example.com:8443/page?key=val");
    }

    #[test]
    fn redirects_to_custom_https_hostname() {
        let req = request_with_host("www.example.com", "/");
        let opts = make_opts("secure.example.com", 443, &["www.example.com"]);
        let resp = redirect_to_https(&req, opts).unwrap();
        let location = resp.headers().get(LOCATION).unwrap().to_str().unwrap();
        assert_eq!(location, "https://secure.example.com:443/");
    }
}
