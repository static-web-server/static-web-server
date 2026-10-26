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

    let x_real_ip = x_real_ip(opts, req, remote_addr);
    let real_remote_ip = forwarded_for(opts, req, remote_addr);

    let method = req.method();
    let uri = req.uri();

    // Log incoming requests in debug mode only if the health option is enabled
    if opts.health && health::is_health_endpoint(uri) {
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

/// Whether the `X-Real-IP` and `X-Forwarded-For` headers of a request from `remote_addr` are trusted.
fn is_trusted_proxy(opts: &RequestHandlerOpts, remote_addr: Option<SocketAddr>) -> bool {
    opts.trusted_proxies.is_empty()
        || remote_addr.is_some_and(|addr| opts.trusted_proxies.contains(&addr.ip()))
}

/// The `X-Real-IP` address, if logging it is enabled and the request comes from a trusted proxy.
pub(crate) fn x_real_ip<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    if !opts.log_x_real_ip || !is_trusted_proxy(opts, remote_addr) {
        return None;
    }
    req.headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
}

/// The first `X-Forwarded-For` address, if logging it is enabled and the request comes from a
/// trusted proxy.
pub(crate) fn forwarded_for<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    if !opts.log_forwarded_for || !is_trusted_proxy(opts, remote_addr) {
        return None;
    }
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn request(header: &str, value: &str) -> Request<()> {
        Request::get("/").header(header, value).body(()).unwrap()
    }

    #[test]
    fn x_real_ip_trusted() {
        let req = request("X-Real-IP", "10.0.0.1");
        let opts = RequestHandlerOpts {
            log_x_real_ip: true,
            ..Default::default()
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        let ip = x_real_ip(&opts, &req, Some(addr));
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn x_real_ip_untrusted() {
        let req = request("X-Real-IP", "10.0.0.1");
        let opts = RequestHandlerOpts {
            log_x_real_ip: true,
            trusted_proxies: vec![IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))],
            ..Default::default()
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        assert_eq!(x_real_ip(&opts, &req, Some(addr)), None);
    }

    #[test]
    fn x_real_ip_disabled() {
        let req = request("X-Real-IP", "10.0.0.1");
        let opts = RequestHandlerOpts::default();
        assert_eq!(x_real_ip(&opts, &req, None), None);
    }

    #[test]
    fn forwarded_for_first_ip() {
        let req = request("X-Forwarded-For", "10.0.0.1, 10.0.0.2");
        let opts = RequestHandlerOpts {
            log_forwarded_for: true,
            ..Default::default()
        };
        let ip = forwarded_for(&opts, &req, None);
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }
}
