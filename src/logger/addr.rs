// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module to log remote and real IP addresses.
//!

use hyper::Request;
use std::net::{IpAddr, SocketAddr};

use crate::handler_opts::RequestHandlerOpts;

/// The target of the lines of this module, kept from its former path for log filters.
pub(super) const TARGET: &str = "static_web_server::log_addr";

/// Initializes the log address module.
pub(crate) fn init(enabled: bool, unix_socket: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.log_remote_address = enabled;
    let trusted = if handler_opts.trusted_proxies.is_empty() {
        "none".to_owned()
    } else {
        format!("{:?}", handler_opts.trusted_proxies)
    };

    tracing::info!(target: TARGET, enabled, "log requests with remote IP addresses");
    tracing::info!(target: TARGET, enabled = handler_opts.log_x_real_ip, "log X-Real-IP header");
    tracing::info!(
        target: TARGET,
        enabled = handler_opts.log_forwarded_for,
        "log X-Forwarded-For header"
    );
    tracing::info!(
        target: TARGET,
        trusted_proxies = %trusted,
        "trusted IPs for X-Real-IP and X-Forwarded-For"
    );

    if let Some(warning) = proxy_headers_warning(handler_opts, unix_socket) {
        tracing::warn!(target: TARGET, "{warning}");
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
    remote_addr.is_some_and(|addr| is_trusted_ip(opts, addr.ip()))
}

/// Whether `ip` is one of the `trusted_proxies`, comparing both in their canonical form.
fn is_trusted_ip(opts: &RequestHandlerOpts, ip: IpAddr) -> bool {
    let ip = ip.to_canonical();
    opts.trusted_proxies.iter().any(|p| p.to_canonical() == ip)
}

/// The `X-Real-IP` address, if the request comes from a trusted proxy.
///
/// The proxy must overwrite the header. If it adds a line instead, the one the client
/// sent can't be told apart from it, so nothing is logged for more than one line.
pub(crate) fn x_real_ip<T>(
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
        tracing::debug!(
            target: TARGET,
            "X-Real-IP not logged, the request has more than one X-Real-IP line"
        );
        return None;
    }
    line.to_str().ok()?.trim().parse().ok()
}

/// The client address from `X-Forwarded-For`, if the request comes from a trusted proxy.
///
/// Each proxy appends the address it got the request from, to the same header line
/// or as a new line, after whatever the client sent. So the entries of all lines are
/// read from the right, skipping `trusted_proxies`, and the first other address is
/// the client. Anything left of it was sent by the client and is ignored.
/// The walk stops with no address at an entry that is empty or not a bare IP address,
/// rather than skip it to reach an address further left. If every entry is a trusted
/// proxy, the leftmost one is returned.
///
/// Lines are split as bytes, so an invalid entry left of the client is never read.
pub(crate) fn forwarded_for<T>(
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
        target: TARGET,
        "X-Forwarded-For not logged, an entry is {reason}: \"{}\" ({} bytes)",
        entry[..entry.len().min(64)].escape_ascii(),
        entry.len(),
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
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
    fn x_real_ip_more_than_one_line_logs_nothing() {
        let proxy = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
        let opts = RequestHandlerOpts {
            trusted_proxies: vec![proxy],
            ..Default::default()
        };
        let addr = SocketAddr::new(proxy, 1234);

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
            assert_eq!(x_real_ip(&opts, &req, Some(addr)), None, "{lines:?}");
        }

        let req = request("X-Real-IP", "6.6.6.6, 10.0.0.1");
        assert_eq!(
            x_real_ip(&opts, &req, Some(addr)),
            None,
            "one combined line"
        );
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

        let format = crate::logger::AccessLogFormat::parse("%{x_real_ip}").ok();
        let custom = RequestHandlerOpts {
            access_log_format: format,
            ..Default::default()
        };
        assert!(proxy_headers_warning(&custom, false).is_some());
    }

    /// The direct peer of the `X-Forwarded-For` tests, always a trusted proxy.
    const PROXY: &str = "172.16.0.1";

    /// The `X-Forwarded-For` address of a request with `lines` from `PROXY`,
    /// trusting `PROXY` and `proxies`.
    fn forwarded_for_lines(lines: &[&[u8]], proxies: &[&str]) -> Option<IpAddr> {
        let mut req = Request::get("/");
        for line in lines {
            req = req.header("X-Forwarded-For", *line);
        }
        let req = req.body(()).unwrap();
        let trusted_proxies = std::iter::once(PROXY)
            .chain(proxies.iter().copied())
            .map(|p| p.parse().unwrap())
            .collect();
        let opts = RequestHandlerOpts {
            trusted_proxies,
            ..Default::default()
        };
        let peer = SocketAddr::new(PROXY.parse().unwrap(), 1234);
        forwarded_for(&opts, &req, Some(peer))
    }

    fn ip(s: &str) -> Option<IpAddr> {
        Some(s.parse().unwrap())
    }

    #[test]
    fn forwarded_for_single_ip() {
        assert_eq!(forwarded_for_lines(&[b"10.0.0.1"], &[]), ip("10.0.0.1"));
    }

    #[test]
    fn forwarded_for_appended_chain_logs_rightmost_untrusted() {
        // nginx `$proxy_add_x_forwarded_for` appends to the line the client sent
        let client = forwarded_for_lines(&[b"6.6.6.6, 203.0.113.50"], &[]);
        assert_eq!(client, ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_reads_all_header_lines() {
        // HAProxy `option forwardfor` adds a line after the one the client sent
        let client = forwarded_for_lines(&[b"6.6.6.6", b"203.0.113.50"], &[]);
        assert_eq!(client, ip("203.0.113.50"));

        let lines: &[&[u8]] = &[b"6.6.6.6, 7.7.7.7", b"203.0.113.50, 10.0.0.2"];
        let client = forwarded_for_lines(lines, &["10.0.0.2"]);
        assert_eq!(client, ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_multi_proxy_chain() {
        let lines: &[&[u8]] = &[b"6.6.6.6, 203.0.113.50, 10.0.0.2, 10.0.0.3"];
        let client = forwarded_for_lines(lines, &["10.0.0.2", "10.0.0.3"]);
        assert_eq!(client, ip("203.0.113.50"));

        let client = forwarded_for_lines(lines, &["10.0.0.3"]);
        assert_eq!(client, ip("10.0.0.2"), "an unlisted proxy is the client");
    }

    #[test]
    fn forwarded_for_all_trusted_logs_leftmost() {
        let lines: &[&[u8]] = &[b"10.0.0.2, 10.0.0.3", b"172.16.0.1"];
        let client = forwarded_for_lines(lines, &["10.0.0.2", "10.0.0.3"]);
        assert_eq!(client, ip("10.0.0.2"));
    }

    #[test]
    fn forwarded_for_malformed_entry_stops_the_walk() {
        let cases: &[&[&[u8]]] = &[
            &[b"6.6.6.6, 203.0.113.50, unknown"],
            &[b"6.6.6.6, 203.0.113.50, 10.0.0.2:8080"],
            &[b"6.6.6.6, 203.0.113.50, [2001:db8::2]"],
            &[b"6.6.6.6, 203.0.113.50, unknown, 10.0.0.2"],
            &[b"6.6.6.6", b"203.0.113.50, bogus"],
            &[b"6.6.6.6, 203.0.113.50", b"10.0.0.\xff"],
            &[b"6.6.6.6, 203.0.113.50", "10.0.0.2\u{e9}".as_bytes()],
            &[b"203.0.113.50, \xff"],
            &[b"203.0.113.50, caf\xc3\xa9, 172.16.0.1"],
        ];
        for lines in cases {
            let client = forwarded_for_lines(lines, &["10.0.0.2"]);
            assert_eq!(client, None, "{lines:?}");
        }
    }

    #[test]
    fn forwarded_for_non_ascii_entry_left_of_client() {
        // nginx `$proxy_add_x_forwarded_for` appends to the client's line, so a non-ASCII
        // entry the client sent ends up on the same line as the client address
        let cases: [&[u8]; 4] = [
            b"\xff, 203.0.113.50",
            b"caf\xc3\xa9, 203.0.113.50",
            b"6.6.6.6, \xff\xfe, 203.0.113.50, 172.16.0.1",
            b"\xff,203.0.113.50",
        ];
        for line in cases {
            let client = forwarded_for_lines(&[line], &[]);
            assert_eq!(client, ip("203.0.113.50"), "{line:?}");
        }
    }

    #[test]
    fn forwarded_for_malformed_entry_left_of_the_client_is_ignored() {
        let lines: &[&[u8]] = &[b"\xff", b"unknown, 203.0.113.50"];
        assert_eq!(forwarded_for_lines(lines, &[]), ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_untrusted_peer_logs_nothing() {
        let req = request("X-Forwarded-For", "6.6.6.6, 203.0.113.50");
        let opts = RequestHandlerOpts {
            trusted_proxies: vec![PROXY.parse().unwrap()],
            ..Default::default()
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        assert_eq!(forwarded_for(&opts, &req, Some(addr)), None);

        // Trusted addresses in the header do not make the peer trusted
        let req = request("X-Forwarded-For", "6.6.6.6, 172.16.0.1");
        assert_eq!(forwarded_for(&opts, &req, Some(addr)), None);
    }

    #[test]
    fn forwarded_for_ipv4_mapped_proxy_entries() {
        let lines: &[&[u8]] = &[b"6.6.6.6, 203.0.113.50, ::ffff:10.0.0.2"];
        let client = forwarded_for_lines(lines, &["10.0.0.2"]);
        assert_eq!(client, ip("203.0.113.50"), "mapped entry, IPv4 proxy");

        let lines: &[&[u8]] = &[b"6.6.6.6, 203.0.113.50, 10.0.0.2"];
        let client = forwarded_for_lines(lines, &["::ffff:10.0.0.2"]);
        assert_eq!(client, ip("203.0.113.50"), "IPv4 entry, mapped proxy");

        let lines: &[&[u8]] = &[b"6.6.6.6, ::ffff:203.0.113.50, 10.0.0.2"];
        let client = forwarded_for_lines(lines, &["10.0.0.2"]);
        assert_eq!(client, ip("::ffff:203.0.113.50"), "logged as sent");

        let lines: &[&[u8]] = &[b"6.6.6.6, 203.0.113.50, ::10.0.0.2"];
        let client = forwarded_for_lines(lines, &["10.0.0.2"]);
        assert_eq!(client, ip("::10.0.0.2"), "IPv4-compatible is not IPv4");
    }

    #[test]
    fn forwarded_for_ipv6_chain() {
        let lines: &[&[u8]] = &[b"2001:db8::6, 2001:db8::50, fd00::2"];
        let client = forwarded_for_lines(lines, &["fd00::2"]);
        assert_eq!(client, ip("2001:db8::50"));
    }

    #[test]
    fn forwarded_for_whitespace_around_entries() {
        let lines: &[&[u8]] = &[b" 6.6.6.6 ,\t203.0.113.50 ", b" 10.0.0.2\t"];
        let client = forwarded_for_lines(lines, &["10.0.0.2"]);
        assert_eq!(client, ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_empty_entry_left_of_client_is_never_reached() {
        let lines: &[&[u8]] = &[b"6.6.6.6,,, 203.0.113.50"];
        assert_eq!(forwarded_for_lines(lines, &[]), ip("203.0.113.50"));

        let lines: &[&[u8]] = &[b"", b" , ", b"203.0.113.50, 10.0.0.2"];
        let client = forwarded_for_lines(lines, &["10.0.0.2"]);
        assert_eq!(client, ip("203.0.113.50"));
    }

    #[test]
    fn forwarded_for_empty_entry_right_of_client_logs_nothing() {
        let trusted = &["10.0.0.2"];
        // A trusted proxy that appends an empty value, to the same line or as a new one
        assert_eq!(forwarded_for_lines(&[b"6.6.6.6, "], &[]), None);
        assert_eq!(forwarded_for_lines(&[b"6.6.6.6", b""], &[]), None);
        assert_eq!(forwarded_for_lines(&[b"6.6.6.6,, 10.0.0.2"], trusted), None);
        assert_eq!(
            forwarded_for_lines(&[b"6.6.6.6", b" ", b"10.0.0.2"], trusted),
            None
        );

        assert_eq!(forwarded_for_lines(&[b""], &[]), None);
        assert_eq!(forwarded_for_lines(&[b" , ,"], &[]), None);
        assert_eq!(forwarded_for_lines(&[], &[]), None);
    }

    /// Inner proxies the property test chains through, all trusted.
    const INNER_PROXIES: [&str; 4] = ["10.0.0.2", "10.0.0.3", "::ffff:10.0.0.4", "fd00::5"];

    /// A header line of any bytes a header value allows: tab, visible ASCII and obs-text.
    fn header_line() -> impl Strategy<Value = Vec<u8>> {
        let byte = prop_oneof![Just(b'\t'), 0x20u8..=0x7e, 0x80u8..=0xff];
        proptest::collection::vec(byte, 0..32)
    }

    proptest! {
        /// Whatever the client sends, the address the trusted proxy chain appends for it is
        /// logged, whether each proxy appends to the last line or adds a new one.
        #[test]
        fn prop_forwarded_for_ignores_what_the_client_sent(
            sent in proptest::collection::vec(header_line(), 0..4),
            client in proptest::sample::select(
                vec!["203.0.113.50", "2001:db8::50", "::ffff:203.0.113.50"],
            ),
            inner in proptest::sample::subsequence(INNER_PROXIES.to_vec(), 0..=INNER_PROXIES.len()),
            same_line in proptest::collection::vec(any::<bool>(), INNER_PROXIES.len() + 1),
        ) {
            // The first proxy appends the client, each next one the proxy before it
            let mut lines = sent;
            for (hop, same_line) in std::iter::once(client).chain(inner).zip(same_line) {
                match lines.last_mut() {
                    Some(last) if same_line => last.extend(format!(", {hop}").into_bytes()),
                    _ => lines.push(hop.as_bytes().to_vec()),
                }
            }
            let lines: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();

            prop_assert_eq!(forwarded_for_lines(&lines, &INNER_PROXIES), ip(client));
        }
    }
}
