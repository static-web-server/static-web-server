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
        "none".to_owned()
    } else {
        format!("{:?}", handler_opts.trusted_proxies)
    };

    tracing::info!("log requests with remote IP addresses: enabled={enabled}");
    tracing::info!(
        "log X-Real-IP header: enabled={}",
        handler_opts.log_x_real_ip
    );
    tracing::info!(
        "log X-Forwarded-For header: enabled={}",
        handler_opts.log_forwarded_for
    );
    tracing::info!("trusted IPs for X-Real-IP and X-Forwarded-For: {trusted}");

    if let Some(warning) = proxy_headers_warning(handler_opts) {
        tracing::warn!("{warning}");
    }
}

/// Why the `X-Real-IP` and `X-Forwarded-For` headers will never be logged, if they are set to be.
fn proxy_headers_warning(opts: &RequestHandlerOpts) -> Option<&'static str> {
    if (opts.log_x_real_ip || opts.log_forwarded_for) && opts.trusted_proxies.is_empty() {
        Some(
            "X-Real-IP and X-Forwarded-For are not logged since no trusted proxies are set, use --trusted-proxies",
        )
    } else {
        None
    }
}

/// Whether the `X-Real-IP` and `X-Forwarded-For` headers of a request from `remote_addr` are trusted.
///
/// Only requests from `trusted_proxies` are, so an empty list trusts nobody.
/// IPv4-mapped IPv6 addresses (`::ffff:a.b.c.d`) of a dual-stack listener match their IPv4 entry.
fn is_trusted_proxy(opts: &RequestHandlerOpts, remote_addr: Option<SocketAddr>) -> bool {
    remote_addr.is_some_and(|addr| is_trusted_ip(opts, addr.ip()))
}

/// Whether `ip` is in `trusted_proxies`, matching IPv4-mapped IPv6 addresses to their IPv4 entry.
fn is_trusted_ip(opts: &RequestHandlerOpts, ip: IpAddr) -> bool {
    let ip = ip.to_canonical();
    opts.trusted_proxies.iter().any(|p| p.to_canonical() == ip)
}

/// The `X-Real-IP` address, if the request comes from a trusted proxy.
///
/// The proxy must overwrite the header. If it adds a line instead, the one the client
/// sent can't be told apart from it, so nothing is logged for more than one line.
fn x_real_ip<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    if !is_trusted_proxy(opts, remote_addr) {
        return None;
    }
    let mut lines = req.headers().get_all("X-Real-IP").iter();
    let line = lines.next()?;
    if lines.next().is_some() {
        tracing::debug!("X-Real-IP not logged, the request has more than one X-Real-IP line");
        return None;
    }
    line.to_str().ok()?.trim().parse().ok()
}

/// The client address in `X-Forwarded-For`, if the request comes from a trusted proxy.
///
/// Proxies append to the header, so only the entries on the right are known to be set by
/// trusted proxies. The entries of all header lines are walked from the right, skipping
/// trusted proxies, and the first untrusted address is the client. An empty or invalid entry
/// stops the walk with no address, since anything to its left may be set by the client. If
/// every entry is a trusted proxy, the leftmost one is the client.
///
/// Lines are split as bytes, so an invalid entry left of the client is never read.
fn forwarded_for<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    if !is_trusted_proxy(opts, remote_addr) {
        return None;
    }

    let mut leftmost = None;
    for line in req.headers().get_all("X-Forwarded-For").iter().rev() {
        let entries = line.as_bytes().rsplit(|b| *b == b',');
        for entry in entries.map(<[u8]>::trim_ascii) {
            let ip = forwarded_for_ip(entry)?;
            if !is_trusted_ip(opts, ip) {
                return Some(ip);
            }
            leftmost = Some(ip);
        }
    }
    leftmost
}

/// Parses an `X-Forwarded-For` entry as a bare IP address.
fn forwarded_for_ip(entry: &[u8]) -> Option<IpAddr> {
    let reason = match std::str::from_utf8(entry) {
        Ok("") => "empty",
        Ok(s) => match s.parse() {
            Ok(ip) => return Some(ip),
            Err(_) => "not an IP address",
        },
        Err(_) => "not valid UTF-8",
    };
    tracing::debug!(
        "X-Forwarded-For not logged, an entry is {reason}: \"{}\" ({} bytes)",
        entry[..entry.len().min(64)].escape_ascii(),
        entry.len(),
    );
    None
}

/// It logs remote and real IP addresses if available.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) {
    let remote_addrs = remote_addrs(opts, req, remote_addr);

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

/// The remote and real IP address fields of the incoming request log line.
fn remote_addrs<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    remote_addr: Option<SocketAddr>,
) -> String {
    let mut remote_addrs = String::new();

    if opts.log_remote_address
        && let Some(addr) = remote_addr
    {
        remote_addrs.push_str(format!(" remote_addr={addr}").as_str());
    }
    if opts.log_x_real_ip
        && let Some(real_ip) = x_real_ip(opts, req, remote_addr)
    {
        remote_addrs.push_str(format!(" x_real_ip={real_ip}").as_str());
    }
    if opts.log_forwarded_for
        && let Some(forwarded_for) = forwarded_for(opts, req, remote_addr)
    {
        remote_addrs.push_str(format!(" real_remote_ip={forwarded_for}").as_str());
    }

    remote_addrs
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::HeaderValue;
    use std::net::{Ipv4Addr, Ipv6Addr};

    const PROXY: IpAddr = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
    const CLIENT: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

    fn request(header: &str, value: &str) -> Request<()> {
        Request::get("/").header(header, value).body(()).unwrap()
    }

    fn trusting(trusted_proxies: Vec<IpAddr>) -> RequestHandlerOpts {
        RequestHandlerOpts {
            trusted_proxies,
            ..Default::default()
        }
    }

    fn from(ip: IpAddr) -> Option<SocketAddr> {
        Some(SocketAddr::new(ip, 1234))
    }

    #[test]
    fn x_real_ip_no_trusted_proxies_trusts_nobody() {
        let req = request("X-Real-IP", "10.0.0.1");
        let opts = RequestHandlerOpts::default();
        assert_eq!(x_real_ip(&opts, &req, from(PROXY)), None);
    }

    #[test]
    fn forwarded_for_no_trusted_proxies_trusts_nobody() {
        let req = request("X-Forwarded-For", "10.0.0.1");
        let opts = RequestHandlerOpts::default();
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), None);
    }

    #[test]
    fn forwarded_for_without_remote_addr_is_untrusted() {
        let req = request("X-Forwarded-For", "10.0.0.1");
        assert_eq!(forwarded_for(&trusting(vec![PROXY]), &req, None), None);
    }

    #[test]
    fn x_real_ip_untrusted_proxy() {
        let req = request("X-Real-IP", "10.0.0.1");
        let other = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(x_real_ip(&trusting(vec![PROXY]), &req, from(other)), None);
    }

    #[test]
    fn x_real_ip_trusted_proxy() {
        let req = request("X-Real-IP", " 10.0.0.1 ");
        let ip = x_real_ip(&trusting(vec![PROXY]), &req, from(PROXY));
        assert_eq!(ip, Some(CLIENT));
    }

    #[test]
    fn x_real_ip_more_than_one_line_logs_nothing() {
        let opts = trusting(vec![PROXY]);
        for lines in [
            ["6.6.6.6", "10.0.0.1"],
            ["10.0.0.1", "6.6.6.6"],
            ["10.0.0.1", ""],
        ] {
            let req = Request::get("/")
                .header("X-Real-IP", lines[0])
                .header("X-Real-IP", lines[1])
                .body(())
                .unwrap();
            assert_eq!(x_real_ip(&opts, &req, from(PROXY)), None, "{lines:?}");
        }

        let req = request("X-Real-IP", "6.6.6.6, 10.0.0.1");
        assert_eq!(
            x_real_ip(&opts, &req, from(PROXY)),
            None,
            "one combined line"
        );
    }

    #[test]
    fn forwarded_for_trusted_proxy_rightmost_ip() {
        let req = request("X-Forwarded-For", "10.0.0.2, 10.0.0.1");
        let ip = forwarded_for(&trusting(vec![PROXY]), &req, from(PROXY));
        assert_eq!(ip, Some(CLIENT));
    }

    #[test]
    fn trusted_proxy_ipv4_mapped() {
        let req = request("X-Real-IP", "10.0.0.1");
        let mapped = IpAddr::V6(Ipv4Addr::new(172, 16, 0, 1).to_ipv6_mapped());

        // IPv4 client of a dual-stack (`::`) listener, IPv4 entry
        let ip = x_real_ip(&trusting(vec![PROXY]), &req, from(mapped));
        assert_eq!(ip, Some(CLIENT));

        // IPv4 client of an IPv4 listener, IPv4-mapped entry
        let ip = x_real_ip(&trusting(vec![mapped]), &req, from(PROXY));
        assert_eq!(ip, Some(CLIENT));

        // Another IPv4-mapped address is still untrusted
        let other = IpAddr::V6(Ipv4Addr::new(172, 16, 0, 2).to_ipv6_mapped());
        assert_eq!(x_real_ip(&trusting(vec![PROXY]), &req, from(other)), None);
    }

    #[test]
    fn trusted_proxy_ipv6() {
        let req = request("X-Forwarded-For", "2001:db8::1");
        let proxy = IpAddr::V6(Ipv6Addr::LOCALHOST);
        let ip = forwarded_for(&trusting(vec![proxy]), &req, from(proxy));
        assert_eq!(ip, Some("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn proxy_headers_warning_when_never_logged() {
        let opts = |log_x_real_ip, log_forwarded_for, trusted_proxies| RequestHandlerOpts {
            log_x_real_ip,
            log_forwarded_for,
            trusted_proxies,
            ..Default::default()
        };

        assert!(proxy_headers_warning(&opts(false, false, vec![])).is_none());
        assert!(proxy_headers_warning(&opts(true, true, vec![PROXY])).is_none());
        assert!(proxy_headers_warning(&opts(true, false, vec![])).is_some());
        assert!(proxy_headers_warning(&opts(false, true, vec![])).is_some());
    }

    #[test]
    fn trusted_proxy_ipv4_compatible_is_not_ipv4() {
        let req = request("X-Real-IP", "10.0.0.1");
        // `::172.16.0.1` is IPv4-compatible, not IPv4-mapped, so it is a different address
        let compatible = IpAddr::V6(Ipv4Addr::new(172, 16, 0, 1).to_ipv6_compatible());

        assert_eq!(
            x_real_ip(&trusting(vec![PROXY]), &req, from(compatible)),
            None
        );
        assert_eq!(
            x_real_ip(&trusting(vec![compatible]), &req, from(PROXY)),
            None
        );
    }

    #[test]
    fn trusted_proxy_ipv6_loopback_is_not_ipv4() {
        let req = request("X-Real-IP", "10.0.0.1");
        let loopback = IpAddr::V6(Ipv6Addr::LOCALHOST);

        for ipv4 in [Ipv4Addr::new(0, 0, 0, 1), Ipv4Addr::LOCALHOST] {
            let ipv4 = IpAddr::V4(ipv4);
            assert_eq!(x_real_ip(&trusting(vec![ipv4]), &req, from(loopback)), None);
            assert_eq!(x_real_ip(&trusting(vec![loopback]), &req, from(ipv4)), None);
        }
    }

    /// A request with both proxy headers set.
    fn proxied_request() -> Request<()> {
        Request::get("/")
            .header("X-Real-IP", "10.0.0.1")
            .header("X-Forwarded-For", "10.0.0.2, 10.0.0.3")
            .body(())
            .unwrap()
    }

    fn logging(
        log_remote_address: bool,
        log_x_real_ip: bool,
        log_forwarded_for: bool,
        trusted_proxies: Vec<IpAddr>,
    ) -> RequestHandlerOpts {
        RequestHandlerOpts {
            log_remote_address,
            log_x_real_ip,
            log_forwarded_for,
            trusted_proxies,
            ..Default::default()
        }
    }

    #[test]
    fn remote_addrs_gates() {
        let req = proxied_request();
        let addrs = |opts: RequestHandlerOpts| remote_addrs(&opts, &req, from(PROXY));

        assert_eq!(addrs(logging(false, false, false, vec![PROXY])), "");
        assert_eq!(
            addrs(logging(true, false, false, vec![PROXY])),
            " remote_addr=172.16.0.1:1234"
        );
        assert_eq!(
            addrs(logging(false, true, false, vec![PROXY])),
            " x_real_ip=10.0.0.1"
        );
        assert_eq!(
            addrs(logging(false, false, true, vec![PROXY])),
            " real_remote_ip=10.0.0.3"
        );
        assert_eq!(
            addrs(logging(true, true, true, vec![PROXY])),
            " remote_addr=172.16.0.1:1234 x_real_ip=10.0.0.1 real_remote_ip=10.0.0.3"
        );
    }

    #[test]
    fn remote_addrs_no_trusted_proxies() {
        let req = proxied_request();
        let opts = logging(true, true, true, vec![]);
        assert_eq!(
            remote_addrs(&opts, &req, from(PROXY)),
            " remote_addr=172.16.0.1:1234"
        );
        assert_eq!(
            remote_addrs(&logging(false, true, true, vec![]), &req, None),
            ""
        );
    }

    #[test]
    fn remote_addrs_untrusted_peer() {
        let req = proxied_request();
        let opts = logging(true, true, true, vec![PROXY]);
        let other = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(
            remote_addrs(&opts, &req, from(other)),
            " remote_addr=192.168.1.1:1234"
        );
    }

    #[test]
    fn remote_addrs_ipv4_mapped_trusted_peer() {
        let req = proxied_request();
        let opts = logging(true, true, true, vec![PROXY]);
        let mapped = IpAddr::V6(Ipv4Addr::new(172, 16, 0, 1).to_ipv6_mapped());
        assert_eq!(
            remote_addrs(&opts, &req, from(mapped)),
            " remote_addr=[::ffff:172.16.0.1]:1234 x_real_ip=10.0.0.1 real_remote_ip=10.0.0.3"
        );
    }

    #[test]
    fn remote_addrs_forwarded_for_reads_all_header_lines() {
        let opts = logging(false, false, true, vec![PROXY]);

        let req = Request::get("/")
            .header("X-Forwarded-For", "10.0.0.2")
            .header("X-Forwarded-For", "10.0.0.3")
            .body(())
            .unwrap();
        assert_eq!(
            remote_addrs(&opts, &req, from(PROXY)),
            " real_remote_ip=10.0.0.3"
        );

        // An invalid line left of the client is never reached
        let req = Request::get("/")
            .header("X-Forwarded-For", "unknown")
            .header("X-Forwarded-For", "10.0.0.3")
            .body(())
            .unwrap();
        assert_eq!(
            remote_addrs(&opts, &req, from(PROXY)),
            " real_remote_ip=10.0.0.3"
        );
    }

    #[test]
    fn remote_addrs_invalid_entries() {
        let opts = logging(false, true, true, vec![PROXY]);

        // An invalid entry left of the client is never reached
        let req = Request::get("/")
            .header("X-Real-IP", "not-an-ip")
            .header("X-Forwarded-For", "unknown, 10.0.0.3")
            .body(())
            .unwrap();
        assert_eq!(
            remote_addrs(&opts, &req, from(PROXY)),
            " real_remote_ip=10.0.0.3"
        );

        // An invalid entry right of the client logs nothing
        let req = Request::get("/")
            .header("X-Real-IP", "not-an-ip")
            .header("X-Forwarded-For", "10.0.0.3, unknown")
            .body(())
            .unwrap();
        assert_eq!(remote_addrs(&opts, &req, from(PROXY)), "");
    }

    fn forwarded_lines(lines: &[&[u8]]) -> Request<()> {
        let mut req = Request::get("/");
        for line in lines {
            req = req.header("X-Forwarded-For", HeaderValue::from_bytes(line).unwrap());
        }
        req.body(()).unwrap()
    }

    fn ip(s: &str) -> Option<IpAddr> {
        Some(s.parse().unwrap())
    }

    #[test]
    fn forwarded_for_appended_chain() {
        // nginx `$proxy_add_x_forwarded_for` appends the client address to its header
        let req = request("X-Forwarded-For", "6.6.6.6, 203.0.113.50");
        let opts = trusting(vec![PROXY]);
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_two_header_lines() {
        // HAProxy `option forwardfor` adds a line after the client's own
        let req = forwarded_lines(&[b"6.6.6.6", b"203.0.113.50"]);
        let opts = trusting(vec![PROXY]);
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("203.0.113.50"));

        let req = forwarded_lines(&[b"6.6.6.6, 7.7.7.7", b"203.0.113.50, 172.16.0.2"]);
        let opts = trusting(vec![PROXY, "172.16.0.2".parse().unwrap()]);
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_trusted_proxy_chain() {
        let req = forwarded_lines(&[b"6.6.6.6, 203.0.113.50, 172.16.0.3", b"172.16.0.2"]);
        let chain = vec![
            PROXY,
            "172.16.0.2".parse().unwrap(),
            "172.16.0.3".parse().unwrap(),
        ];
        let ip_of = |opts| forwarded_for(&trusting(opts), &req, from(PROXY));
        assert_eq!(ip_of(chain), ip("203.0.113.50"));

        // An unlisted proxy in the chain is logged as the client
        assert_eq!(
            ip_of(vec![PROXY, "172.16.0.2".parse().unwrap()]),
            ip("172.16.0.3")
        );
    }

    #[test]
    fn forwarded_for_all_trusted_is_leftmost() {
        let req = forwarded_lines(&[b"172.16.0.3, 172.16.0.2", b"172.16.0.1"]);
        let opts = trusting(vec![
            PROXY,
            "172.16.0.2".parse().unwrap(),
            "172.16.0.3".parse().unwrap(),
        ]);
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("172.16.0.3"));
    }

    #[test]
    fn forwarded_for_invalid_entry_right_of_client() {
        let opts = trusting(vec![PROXY]);
        let cases: [&[&[u8]]; 7] = [
            &[b"203.0.113.50, unknown"],
            &[b"203.0.113.50, 172.16.0.1:8080"],
            &[b"203.0.113.50", b"unknown, 172.16.0.1"],
            &[b"203.0.113.50", b"\xff172.16.0.1"],
            &[b"203.0.113.50", "172.16.0.1\u{e9}".as_bytes()],
            &[b"203.0.113.50, \xff"],
            &[b"203.0.113.50, caf\xc3\xa9, 172.16.0.1"],
        ];
        for lines in cases {
            let req = forwarded_lines(lines);
            assert_eq!(forwarded_for(&opts, &req, from(PROXY)), None, "{lines:?}");
        }

        // A non-UTF-8 line left of the client is never reached
        let req = forwarded_lines(&[b"\xff", b"203.0.113.50"]);
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_non_ascii_entry_left_of_client() {
        // nginx `$proxy_add_x_forwarded_for` appends to the client's line, so a non-ASCII
        // entry the client sent ends up on the same line as the client address
        let opts = trusting(vec![PROXY]);
        let cases: [&[u8]; 4] = [
            b"\xff, 203.0.113.50",
            b"caf\xc3\xa9, 203.0.113.50",
            b"6.6.6.6, \xff\xfe, 203.0.113.50, 172.16.0.1",
            b"\xff,203.0.113.50",
        ];
        for line in cases {
            let req = forwarded_lines(&[line]);
            let ip_of = forwarded_for(&opts, &req, from(PROXY));
            assert_eq!(ip_of, ip("203.0.113.50"), "{line:?}");
        }
    }

    #[test]
    fn forwarded_for_untrusted_peer() {
        let req = request("X-Forwarded-For", "6.6.6.6, 203.0.113.50");
        let other = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let opts = trusting(vec![PROXY]);
        assert_eq!(forwarded_for(&opts, &req, from(other)), None);

        // Trusted addresses in the header do not make the peer trusted
        let req = request("X-Forwarded-For", "6.6.6.6, 172.16.0.1");
        assert_eq!(forwarded_for(&opts, &req, from(other)), None);
    }

    #[test]
    fn forwarded_for_ipv4_mapped_proxy_entries() {
        let opts = trusting(vec![PROXY, "172.16.0.2".parse().unwrap()]);

        // IPv4-mapped entries match IPv4 trusted proxies
        let req = request(
            "X-Forwarded-For",
            "6.6.6.6, 203.0.113.50, ::ffff:172.16.0.2, ::ffff:172.16.0.1",
        );
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("203.0.113.50"));

        // IPv4 entries match IPv4-mapped trusted proxies
        let mapped = IpAddr::V6(Ipv4Addr::new(172, 16, 0, 2).to_ipv6_mapped());
        let opts = trusting(vec![PROXY, mapped]);
        let req = request("X-Forwarded-For", "6.6.6.6, 203.0.113.50, 172.16.0.2");
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("203.0.113.50"));

        // IPv4-compatible entries are not IPv4 proxies
        let req = request("X-Forwarded-For", "6.6.6.6, 203.0.113.50, ::172.16.0.2");
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("::172.16.0.2"));
    }

    #[test]
    fn forwarded_for_whitespace_around_entries() {
        let opts = trusting(vec![PROXY]);
        let req = forwarded_lines(&[b"  6.6.6.6 ,\t203.0.113.50 , 172.16.0.1  "]);
        assert_eq!(forwarded_for(&opts, &req, from(PROXY)), ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_empty_entry_left_of_client_is_never_reached() {
        let opts = trusting(vec![PROXY]);
        let ip_of = |lines: &[&[u8]]| forwarded_for(&opts, &forwarded_lines(lines), from(PROXY));

        assert_eq!(ip_of(&[b"6.6.6.6,,, 203.0.113.50"]), ip("203.0.113.50"));
        assert_eq!(
            ip_of(&[b"", b" , ", b"203.0.113.50, 172.16.0.1"]),
            ip("203.0.113.50")
        );
    }

    #[test]
    fn forwarded_for_empty_entry_right_of_client_logs_nothing() {
        let opts = trusting(vec![PROXY]);
        let ip_of = |lines: &[&[u8]]| forwarded_for(&opts, &forwarded_lines(lines), from(PROXY));

        // A trusted proxy that appends an empty value, to the same line or as a new one
        assert_eq!(ip_of(&[b"6.6.6.6, "]), None);
        assert_eq!(ip_of(&[b"6.6.6.6", b""]), None);
        assert_eq!(ip_of(&[b"6.6.6.6,, 172.16.0.1"]), None);
        assert_eq!(ip_of(&[b"6.6.6.6", b" ", b"172.16.0.1"]), None);

        assert_eq!(ip_of(&[b""]), None);
        assert_eq!(ip_of(&[b" , ,"]), None);
        assert_eq!(ip_of(&[]), None);
    }
}
