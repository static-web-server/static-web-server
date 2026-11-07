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
pub(crate) fn init(enabled: bool, unix_socket: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.log_remote_address = enabled;
    let trusted = if handler_opts.trusted_proxies.is_empty() {
        "none".to_owned()
    } else {
        format!("{:?}", handler_opts.trusted_proxies)
    };

    tracing::info!(enabled, "log requests with remote IP addresses");
    tracing::info!(enabled = handler_opts.log_x_real_ip, "log X-Real-IP header");
    tracing::info!(
        enabled = handler_opts.log_forwarded_for,
        "log X-Forwarded-For header"
    );
    tracing::info!(
        trusted_proxies = %trusted,
        "trusted IPs for X-Real-IP and X-Forwarded-For"
    );

    if let Some(warning) = proxy_headers_warning(handler_opts, unix_socket) {
        tracing::warn!("{warning}");
    }
}

/// Why the `X-Real-IP` and `X-Forwarded-For` headers will never be logged, if they are set to be.
fn proxy_headers_warning(opts: &RequestHandlerOpts, unix_socket: bool) -> Option<&'static str> {
    if !logs_proxy_headers(opts) {
        None
    } else if unix_socket {
        Some(
            "X-Real-IP and X-Forwarded-For are not logged since Unix socket peers have no IP address",
        )
    } else if opts.trusted_proxies.is_empty() {
        Some(
            "X-Real-IP and X-Forwarded-For are not logged since no trusted proxies are set, use --trusted-proxies",
        )
    } else {
        None
    }
}

/// Whether the access log is set to include `X-Real-IP` or `X-Forwarded-For`.
fn logs_proxy_headers(opts: &RequestHandlerOpts) -> bool {
    opts.log_x_real_ip
        || opts.log_forwarded_for
        || opts
            .access_log_format
            .as_ref()
            .is_some_and(|f| f.uses_proxy_headers())
}

/// Whether the `X-Real-IP` and `X-Forwarded-For` headers of a request from `remote_addr` are trusted.
///
/// Only requests from `trusted_proxies` are, so an empty list trusts nobody.
/// IPv4-mapped IPv6 addresses (`::ffff:a.b.c.d`) of a dual-stack listener match their IPv4 entry.
fn is_trusted_proxy(opts: &RequestHandlerOpts, remote_addr: Option<SocketAddr>) -> bool {
    remote_addr.is_some_and(|addr| {
        let ip = addr.ip().to_canonical();
        opts.trusted_proxies.iter().any(|p| p.to_canonical() == ip)
    })
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
    fn x_real_ip_no_trusted_proxies_trusts_nobody() {
        let req = request("X-Real-IP", "10.0.0.1");
        let opts = RequestHandlerOpts::default();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        assert_eq!(x_real_ip(&opts, &req, Some(addr)), None);
    }

    #[test]
    fn forwarded_for_no_trusted_proxies_trusts_nobody() {
        let req = request("X-Forwarded-For", "10.0.0.1");
        let opts = RequestHandlerOpts::default();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        assert_eq!(forwarded_for(&opts, &req, Some(addr)), None);
    }

    #[test]
    fn forwarded_for_without_remote_addr_is_untrusted() {
        let req = request("X-Forwarded-For", "10.0.0.1");
        let opts = RequestHandlerOpts {
            trusted_proxies: vec![IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))],
            ..Default::default()
        };
        assert_eq!(forwarded_for(&opts, &req, None), None);
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
    fn x_real_ip_trusted_proxy_ipv4_mapped() {
        let req = request("X-Real-IP", "10.0.0.1");
        let opts = RequestHandlerOpts {
            trusted_proxies: vec![IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))],
            ..Default::default()
        };
        let mapped = Ipv4Addr::new(172, 16, 0, 1).to_ipv6_mapped();
        let addr = SocketAddr::new(IpAddr::V6(mapped), 1234);
        let ip = x_real_ip(&opts, &req, Some(addr));
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));

        let opts = RequestHandlerOpts {
            trusted_proxies: vec![IpAddr::V6(mapped)],
            ..Default::default()
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)), 1234);
        let ip = x_real_ip(&opts, &req, Some(addr));
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn proxy_headers_warning_when_never_logged() {
        let proxy = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
        let opts = |log_forwarded_for, trusted_proxies| RequestHandlerOpts {
            log_forwarded_for,
            trusted_proxies,
            ..Default::default()
        };

        assert!(proxy_headers_warning(&opts(false, vec![]), false).is_none());
        assert!(proxy_headers_warning(&opts(true, vec![proxy]), false).is_none());
        let no_proxies = proxy_headers_warning(&opts(true, vec![]), false);
        assert!(no_proxies.is_some_and(|w| w.contains("no trusted proxies")));
        let unix = proxy_headers_warning(&opts(true, vec![proxy]), true);
        assert!(unix.is_some_and(|w| w.contains("Unix socket")));

        let format = crate::access_logs::AccessLogFormat::parse("%{x_real_ip}").ok();
        let custom = RequestHandlerOpts {
            access_log_format: format,
            ..Default::default()
        };
        assert!(proxy_headers_warning(&custom, false).is_some());
    }

    #[test]
    fn forwarded_for_first_ip() {
        let req = request("X-Forwarded-For", "10.0.0.1, 10.0.0.2");
        let proxy = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
        let opts = RequestHandlerOpts {
            trusted_proxies: vec![proxy],
            ..Default::default()
        };
        let ip = forwarded_for(&opts, &req, Some(SocketAddr::new(proxy, 1234)));
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }
}
