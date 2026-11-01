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

    let x_real_ip = if opts.log_x_real_ip {
        x_real_ip(opts, req, remote_addr)
    } else {
        None
    };
    let real_remote_ip = if opts.log_forwarded_for {
        forwarded_for(opts, req, remote_addr)
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

/// Whether the `X-Real-IP` and `X-Forwarded-For` headers of a request from `remote_addr` are trusted.
fn is_trusted_proxy(opts: &RequestHandlerOpts, remote_addr: Option<SocketAddr>) -> bool {
    opts.trusted_proxies.is_empty()
        || remote_addr.is_some_and(|addr| opts.trusted_proxies.contains(&addr.ip()))
}

/// The `X-Real-IP` address, if the request comes from a trusted proxy.
pub(crate) fn x_real_ip<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    if !is_trusted_proxy(opts, remote_addr) {
        return None;
    }
    req.headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
}

/// The first `X-Forwarded-For` address, if the request comes from a trusted proxy.
pub(crate) fn forwarded_for<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    if !is_trusted_proxy(opts, remote_addr) {
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
        let opts = RequestHandlerOpts::default();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        let ip = x_real_ip(&opts, &req, Some(addr));
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn x_real_ip_untrusted() {
        let req = request("X-Real-IP", "10.0.0.1");
        let opts = RequestHandlerOpts {
            trusted_proxies: vec![IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))],
            ..Default::default()
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        assert_eq!(x_real_ip(&opts, &req, Some(addr)), None);
    }

    #[test]
    fn x_real_ip_trusted_proxy() {
        let req = request("X-Real-IP", "10.0.0.1");
        let proxy = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
        let opts = RequestHandlerOpts {
            trusted_proxies: vec![proxy],
            ..Default::default()
        };
        let addr = SocketAddr::new(proxy, 1234);
        let ip = x_real_ip(&opts, &req, Some(addr));
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn forwarded_for_first_ip() {
        let req = request("X-Forwarded-For", "10.0.0.1, 10.0.0.2");
        let opts = RequestHandlerOpts::default();
        let ip = forwarded_for(&opts, &req, None);
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }
}
