// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module to log remote and real IP addresses.
//!

use hyper::Request;
use std::net::{IpAddr, SocketAddr};

use crate::handler::RequestHandlerOpts;

/// Initializes the log address module.
pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.log_remote_address = enabled;
    server_info!("log requests with remote and real IP addresses: enabled={enabled}");
}

/// It logs remote and real IP addresses if available.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) {
    let remote_addrs = if opts.log_remote_address {
        // Add a Remote IP if available
        let remote_addr = remote_addr.map_or("".to_owned(), |ip| format!(" remote_addr={ip}"));

        // Add also a Real Remote IP if available
        let real_remote_addr = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            .map_or("".to_owned(), |ip| format!(" real_remote_ip={ip}"));

        [remote_addr, real_remote_addr].concat()
    } else {
        String::new()
    };

    // Log incoming requests in debug mode only if the health option is enabled
    if opts.health {
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
