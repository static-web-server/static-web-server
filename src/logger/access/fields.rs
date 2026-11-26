// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The fields of the default `response` line, named after the
//! [OpenTelemetry semantic conventions](https://opentelemetry.io/docs/specs/semconv/http/http-spans/#http-server).
//!
//! Addresses are only logged when their option is enabled: `client.address` is the
//! `X-Forwarded-For` client, else the `X-Real-IP` address, else the remote address, and
//! `network.peer.address` is the remote address. IPv4-mapped IPv6 addresses are logged as IPv4.

use hyper::Uri;
use hyper::header::HeaderValue;
use std::borrow::Cow;
use std::net::IpAddr;

use super::AccessLog;
use super::render::{MAX_ESCAPED_LEN, request_target};

/// The fields of the default `response` line of a request.
#[derive(Debug, PartialEq)]
pub(super) struct ResponseFields<'a> {
    /// `http.request.method`
    pub(super) method: &'a str,
    /// `url.path`
    pub(super) path: &'a str,
    /// `url.query`, without the leading `?`, `None` when empty.
    pub(super) query: Option<&'a str>,
    /// `http.response.status_code`
    pub(super) status: u16,
    /// `http.response.body.size`, `None` when no body is sent (`HEAD`, `204`, `304`).
    pub(super) body_size: Option<u64>,
    /// `duration` in seconds, the value of `%{duration}`.
    pub(super) duration: f64,
    /// `user_agent.original`, cut at [`MAX_ESCAPED_LEN`] bytes.
    pub(super) user_agent: Option<Cow<'a, str>>,
    /// `client.address`
    pub(super) client_address: Option<IpAddr>,
    /// `network.peer.address`
    pub(super) peer_address: Option<IpAddr>,
}

impl<'a> ResponseFields<'a> {
    /// The fields of `log` for a response with `status`, sending `body_size` bytes in `duration`
    /// seconds.
    pub(super) fn new(
        log: &'a AccessLog,
        status: u16,
        body_size: Option<u64>,
        duration: f64,
    ) -> Self {
        let (path, query) = url_path_query(&log.uri);
        Self {
            method: log.method.as_str(),
            path,
            query,
            status,
            body_size,
            duration,
            user_agent: log.user_agent.as_ref().and_then(user_agent_original),
            client_address: log.forwarded_for.or(log.x_real_ip).or(log.remote_ip),
            peer_address: log.remote_ip,
        }
    }
}

/// The `url.path` and `url.query` of `uri`: the request target as `%{uri}` logs it, split at the
/// first `?`. The query is `None` when it is empty.
pub(crate) fn url_path_query(uri: &Uri) -> (&str, Option<&str>) {
    let target = request_target(uri);
    match target.split_once('?') {
        Some((path, query)) => (path, Some(query).filter(|q| !q.is_empty())),
        None => (target, None),
    }
}

/// The `User-Agent` value, `None` when empty. Bytes that are not valid UTF-8 are replaced with
/// `U+FFFD`, and a value longer than [`MAX_ESCAPED_LEN`] UTF-8 bytes (counted before JSON
/// escaping) is cut on a character boundary and ends with `...`.
fn user_agent_original(value: &HeaderValue) -> Option<Cow<'_, str>> {
    let value = String::from_utf8_lossy(value.as_bytes());
    if value.is_empty() {
        return None;
    }
    if value.len() <= MAX_ESCAPED_LEN {
        return Some(value);
    }
    let mut end = MAX_ESCAPED_LEN;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    Some(Cow::Owned(format!("{}...", &value[..end])))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Request;
    use proptest::prelude::*;
    use std::net::SocketAddr;

    use crate::handler_opts::RequestHandlerOpts;

    const PROXY: &str = "172.16.0.1:1234";

    fn ip(s: &str) -> Option<IpAddr> {
        Some(s.parse().unwrap())
    }

    /// The access log of `req` from `remote_addr`, with the default line.
    fn access_log(opts: &RequestHandlerOpts, req: &Request<()>, remote_addr: &str) -> AccessLog {
        AccessLog::new(None, opts, req, Some(remote_addr.parse().unwrap()))
    }

    /// A request through a trusted proxy that sends both proxy headers.
    fn proxied_request() -> Request<()> {
        Request::get("/")
            .header("X-Real-IP", "203.0.113.7")
            .header("X-Forwarded-For", "6.6.6.6, 203.0.113.195")
            .body(())
            .unwrap()
    }

    fn proxy_opts(
        log_remote_address: bool,
        log_x_real_ip: bool,
        log_forwarded_for: bool,
    ) -> RequestHandlerOpts {
        RequestHandlerOpts {
            log_remote_address,
            log_x_real_ip,
            log_forwarded_for,
            trusted_proxies: vec![PROXY.parse::<SocketAddr>().unwrap().ip()],
            ..Default::default()
        }
    }

    #[test]
    fn response_fields_from_a_request() {
        let req = Request::head("https://example.com:8443/a/b.css?v=1&w=2")
            .header("User-Agent", "curl/8.7.1")
            .body(())
            .unwrap();
        let log = access_log(&RequestHandlerOpts::default(), &req, PROXY);

        let fields = ResponseFields::new(&log, 304, None, 0.000412);

        assert_eq!(
            fields,
            ResponseFields {
                method: "HEAD",
                path: "/a/b.css",
                query: Some("v=1&w=2"),
                status: 304,
                body_size: None,
                duration: 0.000412,
                user_agent: Some(Cow::Borrowed("curl/8.7.1")),
                client_address: None,
                peer_address: None,
            }
        );
    }

    #[test]
    fn response_fields_no_address_without_its_option() {
        let req = proxied_request();
        let fields = |opts: &RequestHandlerOpts, remote_addr: &str| {
            let log = access_log(opts, &req, remote_addr);
            let fields = ResponseFields::new(&log, 200, Some(1), 0.0);
            (fields.client_address, fields.peer_address)
        };

        assert_eq!(
            fields(&proxy_opts(false, false, false), PROXY),
            (None, None)
        );
        let untrusted = "192.0.2.1:1234";
        assert_eq!(
            fields(&proxy_opts(false, true, true), untrusted),
            (None, None)
        );
        assert_eq!(
            fields(&proxy_opts(true, true, true), untrusted),
            (ip("192.0.2.1"), ip("192.0.2.1")),
            "the remote address of an untrusted peer is the client"
        );
    }

    #[test]
    fn response_fields_forwarded_for_wins_over_x_real_ip() {
        let req = proxied_request();
        let fields = |opts: &RequestHandlerOpts| {
            let log = access_log(opts, &req, PROXY);
            let fields = ResponseFields::new(&log, 200, Some(1), 0.0);
            (fields.client_address, fields.peer_address)
        };

        assert_eq!(
            fields(&proxy_opts(false, true, true)),
            (ip("203.0.113.195"), None)
        );
        assert_eq!(
            fields(&proxy_opts(false, true, false)),
            (ip("203.0.113.7"), None)
        );
        assert_eq!(
            fields(&proxy_opts(false, false, true)),
            (ip("203.0.113.195"), None)
        );
        assert_eq!(
            fields(&proxy_opts(true, true, true)),
            (ip("203.0.113.195"), ip("172.16.0.1")),
        );
        assert_eq!(
            fields(&proxy_opts(true, false, false)),
            (ip("172.16.0.1"), ip("172.16.0.1")),
        );

        // A header that is not logged falls back to the next address
        let req = Request::get("/")
            .header("X-Real-IP", "203.0.113.7")
            .header("X-Forwarded-For", "unknown")
            .body(())
            .unwrap();
        let log = access_log(&proxy_opts(false, true, true), &req, PROXY);
        let fields = ResponseFields::new(&log, 200, Some(1), 0.0);
        assert_eq!(fields.client_address, ip("203.0.113.7"));
    }

    #[test]
    fn response_fields_ipv4_mapped_addresses_are_unmapped() {
        let req = Request::get("/")
            .header("X-Real-IP", "::ffff:203.0.113.7")
            .header("X-Forwarded-For", "::ffff:203.0.113.195")
            .body(())
            .unwrap();
        let mapped_proxy = "[::ffff:172.16.0.1]:1234";
        let fields = |opts: &RequestHandlerOpts| {
            let log = access_log(opts, &req, mapped_proxy);
            let fields = ResponseFields::new(&log, 200, Some(1), 0.0);
            (fields.client_address, fields.peer_address)
        };

        assert_eq!(
            fields(&proxy_opts(true, false, false)),
            (ip("172.16.0.1"), ip("172.16.0.1"))
        );
        assert_eq!(
            fields(&proxy_opts(true, true, false)),
            (ip("203.0.113.7"), ip("172.16.0.1"))
        );
        assert_eq!(
            fields(&proxy_opts(true, true, true)),
            (ip("203.0.113.195"), ip("172.16.0.1"))
        );

        let req = Request::get("/").body(()).unwrap();
        let opts = proxy_opts(true, false, false);
        let log = access_log(&opts, &req, "[2001:db8::1]:1234");
        let fields = ResponseFields::new(&log, 200, Some(1), 0.0);
        assert_eq!(fields.client_address, ip("2001:db8::1"), "IPv6 kept as is");
    }

    #[test]
    fn url_path_query_splits_the_request_target() {
        let cases = [
            ("/a/b.css", "/a/b.css", None),
            ("/a/b.css?", "/a/b.css", None),
            ("/a/b.css?v=1", "/a/b.css", Some("v=1")),
            ("/?a=1?b=2", "/", Some("a=1?b=2")),
            ("https://127.0.0.1:8443/a?v=1", "/a", Some("v=1")),
            ("http://example.com", "/", None),
            ("example.com:443", "example.com:443", None),
        ];
        for (uri, path, query) in cases {
            let uri = Uri::from_static(uri);
            assert_eq!(url_path_query(&uri), (path, query), "{uri}");
        }
    }

    #[test]
    fn user_agent_original_is_capped() {
        let ua = |value: &[u8]| {
            let value = HeaderValue::from_bytes(value).unwrap();
            user_agent_original(&value).map(Cow::into_owned)
        };

        assert_eq!(ua(b""), None);
        assert_eq!(ua(b"curl/8.7.1").as_deref(), Some("curl/8.7.1"));
        assert_eq!(
            ua(b"caf\xc3\xa9 \xff").as_deref(),
            Some("caf\u{e9} \u{fffd}")
        );

        let max = "a".repeat(MAX_ESCAPED_LEN);
        assert_eq!(ua(max.as_bytes()), Some(max.clone()));
        let long = "a".repeat(MAX_ESCAPED_LEN + 1);
        assert_eq!(ua(long.as_bytes()), Some(format!("{max}...")));

        // Never cut inside a character
        let mut value = "a".repeat(MAX_ESCAPED_LEN - 1).into_bytes();
        value.extend_from_slice("\u{e9}".as_bytes());
        let cut = format!("{}...", "a".repeat(MAX_ESCAPED_LEN - 1));
        assert_eq!(ua(&value), Some(cut));
    }

    proptest! {
        /// The value is cut after the longest prefix of whole characters that fits.
        #[test]
        fn prop_user_agent_original_is_bounded(value in proptest::collection::vec(
            prop_oneof![0x20u8..=0x7e, 0x80u8..=0xff],
            1..2 * MAX_ESCAPED_LEN,
        )) {
            let header = HeaderValue::from_bytes(&value).unwrap();
            let full = String::from_utf8_lossy(&value);
            let logged = user_agent_original(&header).unwrap();

            if full.len() <= MAX_ESCAPED_LEN {
                prop_assert_eq!(&logged, &full);
            } else {
                let kept = logged.strip_suffix("...").expect("a cut value ends with `...`");
                prop_assert!(kept.len() <= MAX_ESCAPED_LEN);
                prop_assert!(full.starts_with(kept));
                let next = full[kept.len()..].chars().next().unwrap();
                prop_assert!(kept.len() + next.len_utf8() > MAX_ESCAPED_LEN);
            }
        }
    }
}
