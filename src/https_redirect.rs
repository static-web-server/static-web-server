// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to redirect HTTP requests to HTTPS.
//!

use headers::{HeaderMapExt, Host};
use hyper::{header::LOCATION, Body, Request, Response, StatusCode};
use std::sync::Arc;

use crate::Result;

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

        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
        resp.headers_mut().insert(LOCATION, url.parse().unwrap());
        return Ok(resp);
    }

    tracing::debug!("redirect host was not determined!");
    Err(StatusCode::BAD_REQUEST)
}
