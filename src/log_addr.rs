// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module to log remote and real IP addresses.
//!

use hyper::{Request, StatusCode, Uri};
use std::net::{IpAddr, SocketAddr};

use crate::{handler::RequestHandlerOpts, health};

/// Initializes the log address module.
pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.log_remote_address = enabled;
    let trusted = if handler_opts.trusted_proxies.is_empty() {
        "all".to_owned()
    } else {
        format!("{:?}", handler_opts.trusted_proxies)
    };

    tracing::info!(enabled, "log requests with remote IP addresses");
    tracing::info!(enabled = handler_opts.log_x_real_ip, "log X-Real-IP header");
    tracing::info!(
        enabled = handler_opts.log_forwarded_for,
        "log X-Forwarded-For header"
    );
    tracing::info!(trusted_proxies = %trusted, "trusted IPs for X-Forwarded-For");
}

/// Logs one line per request with the response status and the remote and real IP addresses.
///
/// Runs after the response is built so that the line carries the status code. It only logs,
/// unlike the `post_process` steps that take and return the response. `uri` is the URI
/// the client sent: rewrites replace the request URI during the pipeline. `status` is `None`
/// when the handler returned an error instead of a response.
pub(crate) fn log_response<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    uri: &Uri,
    remote_addr: Option<SocketAddr>,
    status: Option<StatusCode>,
) {
    let remote_ip = if opts.log_remote_address {
        remote_addr.map(|addr| addr.ip())
    } else {
        None
    };

    let trusted = opts.trusted_proxies.is_empty()
        || remote_addr.is_some_and(|addr| opts.trusted_proxies.contains(&addr.ip()));

    let x_real_ip = if opts.log_x_real_ip && trusted {
        req.headers()
            .get("X-Real-IP")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
    } else {
        None
    };

    let real_remote_ip = if opts.log_forwarded_for && trusted {
        req.headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
    } else {
        None
    };

    let method = req.method();
    let status = status.map(|s| s.as_u16());

    // Log health check requests in debug mode only if the health option is enabled
    if opts.health && health::is_health_endpoint(uri) {
        tracing::debug!(
            method = %method,
            uri = %uri,
            status,
            remote_addr = remote_ip.as_ref().map(tracing::field::display),
            x_real_ip = x_real_ip.as_ref().map(tracing::field::display),
            real_remote_ip = real_remote_ip.as_ref().map(tracing::field::display),
            "response"
        );
        return;
    }

    tracing::info!(
        method = %method,
        uri = %uri,
        status,
        remote_addr = remote_ip.as_ref().map(tracing::field::display),
        x_real_ip = x_real_ip.as_ref().map(tracing::field::display),
        real_remote_ip = real_remote_ip.as_ref().map(tracing::field::display),
        "response"
    );
}
