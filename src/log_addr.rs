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

    server_info!("log requests with remote IP addresses: enabled={enabled}");
    server_info!(
        "log X-Forwarded-For real remote IP addresses: enabled={}",
        handler_opts.log_forwarded_for
    );
    server_info!("trusted IPs for X-Forwarded-For: {trusted}");
}

/// It logs remote and real IP addresses if available.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) {
    let mut remote_addrs = String::new();

    if opts.log_remote_address {
        if let Some(addr) = remote_addr {
            remote_addrs.push_str(format!(" remote_addr={addr}").as_str());
        }
    }
    if opts.log_forwarded_for
        && (opts.trusted_proxies.is_empty()
            || remote_addr.is_some_and(|addr| opts.trusted_proxies.contains(&addr.ip())))
    {
        if let Some(real_ip) = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
        {
            remote_addrs.push_str(format!(" real_remote_ip={real_ip}").as_str());
        }
    }

    // Log incoming requests in debug mode only if the health option is enabled
    if opts.health && health::is_health_endpoint(req) {
        tracing::debug!(
            "incoming request: method={} uri={}{remote_addrs}",
            req.method(),
            req.uri(),
        );
        return;
    }

    tracing::info!(
        "incoming request: method={} uri={}{remote_addrs}",
        req.method(),
        req.uri(),
    );
}
