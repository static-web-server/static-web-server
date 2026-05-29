// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module to log remote and real IP addresses.
//!

use hyper::Request;
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

/// It logs remote and real IP addresses if available.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
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
    let uri = req.uri();

    // Log incoming requests in debug mode only if the health option is enabled
    if opts.health && health::is_health_endpoint(req) {
        tracing::debug!(
            method = %method,
            uri = %uri,
            remote_addr = remote_ip.as_ref().map(tracing::field::display),
            x_real_ip = x_real_ip.as_ref().map(tracing::field::display),
            real_remote_ip = real_remote_ip.as_ref().map(tracing::field::display),
            "incoming request"
        );
        return;
    }

    tracing::info!(
        method = %method,
        uri = %uri,
        remote_addr = remote_ip.as_ref().map(tracing::field::display),
        x_real_ip = x_real_ip.as_ref().map(tracing::field::display),
        real_remote_ip = real_remote_ip.as_ref().map(tracing::field::display),
        "incoming request"
    );
}
