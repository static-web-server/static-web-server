// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Renders the line of a custom access log format.
//!
//! Values taken from the request (URI, `Host`, `User-Agent`, `Referer`) are client-controlled.
//! Bytes outside printable ASCII, `"` and `\` in them are written as `\xHH`, so a client cannot
//! end a quoted field, start a new log line or inject terminal escape sequences. An escaped
//! value longer than 4 KiB is cut and ends with `...`, so a large header cannot make a large line.

use http::uri::Authority;
use hyper::Uri;
use hyper::header::HeaderValue;
use std::fmt::Write;
use std::net::{IpAddr, SocketAddr};
use time::{OffsetDateTime, format_description::BorrowedFormatItem, macros::format_description};

use super::format::{AccessLogFormat, Segment, Token};

/// Renders `format` using the given request/response context.
pub(super) fn render(format: &AccessLogFormat, ctx: &LogContext<'_>) -> String {
    let mut buf = String::with_capacity(256);
    for seg in format.segments.iter() {
        match seg {
            Segment::Literal(s) => buf.push_str(s),
            Segment::Token(token) => render_token(&mut buf, *token, ctx),
        }
    }
    buf
}

/// All data available for access log rendering.
pub(super) struct LogContext<'a> {
    pub(super) method: &'a str,
    pub(super) uri: &'a Uri,
    pub(super) host: Option<&'a HeaderValue>,
    pub(super) version: &'a str,
    pub(super) status: u16,
    pub(super) bytes: u64,
    pub(super) duration_secs: f64,
    pub(super) remote_addr: Option<SocketAddr>,
    pub(super) user_agent: Option<&'a HeaderValue>,
    pub(super) referer: Option<&'a HeaderValue>,
    pub(super) x_real_ip: Option<IpAddr>,
    pub(super) forwarded_for: Option<IpAddr>,
}

fn render_token(buf: &mut String, token: Token, ctx: &LogContext<'_>) {
    match token {
        Token::Method => buf.push_str(ctx.method),
        Token::Uri => push_escaped(buf, request_target(ctx.uri).as_bytes()),
        Token::Version => buf.push_str(ctx.version),
        Token::Status => {
            let _ = write!(buf, "{}", ctx.status);
        }
        Token::Bytes => match ctx.bytes {
            0 => buf.push('-'),
            bytes => {
                let _ = write!(buf, "{bytes}");
            }
        },
        Token::Duration => {
            let _ = write!(buf, "{:.6}", ctx.duration_secs);
        }
        Token::RemoteAddr => push_ip(buf, ctx.remote_addr.map(|addr| addr.ip())),
        Token::XRealIp => push_ip(buf, ctx.x_real_ip),
        Token::ForwardedFor => push_ip(buf, ctx.forwarded_for),
        Token::Host => push_value(buf, request_host(ctx.uri, ctx.host)),
        Token::UserAgent => push_value(buf, ctx.user_agent.map(HeaderValue::as_bytes)),
        Token::Referer => push_value(buf, ctx.referer.map(HeaderValue::as_bytes)),
        Token::Timestamp => {
            // Local time, respecting the TZ environment variable, or UTC if the offset is unknown
            let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
            push_clf_time(buf, now);
        }
    }
}

/// Common Log Format time, e.g. `29/May/2026:23:43:47 +0200`.
const CLF_TIME: &[BorrowedFormatItem<'_>] = format_description!(
    "[day]/[month repr:short]/[year]:[hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"
);

fn push_clf_time(buf: &mut String, time: OffsetDateTime) {
    match time.format(CLF_TIME) {
        Ok(time) => buf.push_str(&time),
        Err(_) => buf.push('-'),
    }
}

fn push_ip(buf: &mut String, ip: Option<IpAddr>) {
    match ip {
        Some(ip) => {
            let _ = write!(buf, "{ip}");
        }
        None => buf.push('-'),
    }
}

/// The host the client asked for, as `virtual_hosts` resolves it: the URI authority (HTTP/2
/// `:authority` or an absolute-form target) without userinfo, else the `Host` header.
fn request_host<'a>(uri: &'a Uri, host: Option<&'a HeaderValue>) -> Option<&'a [u8]> {
    match uri.authority() {
        Some(authority) => Some(host_port(authority).as_bytes()),
        None => host.map(HeaderValue::as_bytes),
    }
}

/// The path and query of `uri`, without the scheme and authority of an absolute-form or
/// HTTP/2 request URI.
pub(super) fn request_target(uri: &Uri) -> &str {
    match (uri.path_and_query(), uri.authority()) {
        (Some(path), _) => path.as_str(),
        // Authority-form `CONNECT` target, e.g. `example.com:443`
        (None, Some(authority)) if uri.scheme().is_none() => host_port(authority),
        _ => "/",
    }
}

/// The authority without userinfo.
fn host_port(authority: &Authority) -> &str {
    let authority = authority.as_str();
    authority.rsplit_once('@').map_or(authority, |(_, h)| h)
}

fn push_value(buf: &mut String, value: Option<&[u8]>) {
    match value {
        Some(value) if !value.is_empty() => push_escaped(buf, value),
        _ => buf.push('-'),
    }
}

/// Maximum length of an escaped value, before the `...` that marks it as cut.
const MAX_ESCAPED_LEN: usize = 4096;

/// Appends a client-controlled value, writing bytes outside printable ASCII, `"` and `\` as `\xHH`.
/// The escaped value is cut at [`MAX_ESCAPED_LEN`] and ends with `...`, never inside an `\xHH`.
fn push_escaped(buf: &mut String, value: &[u8]) {
    let start = buf.len();
    for &b in value {
        let printable = (0x20..=0x7e).contains(&b) && b != b'"' && b != b'\\';
        let len = if printable { 1 } else { 4 };
        if buf.len() - start + len > MAX_ESCAPED_LEN {
            buf.push_str("...");
            return;
        }
        if printable {
            buf.push(b as char);
        } else {
            let _ = write!(buf, "\\x{b:02X}");
        }
    }
}

pub(super) fn http_version_str(version: hyper::Version) -> &'static str {
    match version {
        hyper::Version::HTTP_09 => "HTTP/0.9",
        hyper::Version::HTTP_10 => "HTTP/1.0",
        hyper::Version::HTTP_11 => "HTTP/1.1",
        hyper::Version::HTTP_2 => "HTTP/2",
        hyper::Version::HTTP_3 => "HTTP/3",
        _ => "HTTP/?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::{HOST, REFERER, USER_AGENT};
    use hyper::{HeaderMap, Method};
    use proptest::prelude::*;
    use std::net::Ipv4Addr;
    use time::macros::datetime;

    fn context<'a>(uri: &'a Uri, headers: &'a HeaderMap) -> LogContext<'a> {
        LogContext {
            method: Method::GET.as_str(),
            uri,
            host: headers.get(HOST),
            version: "HTTP/1.1",
            status: 200,
            bytes: 0,
            duration_secs: 0.0,
            remote_addr: None,
            user_agent: headers.get(USER_AGENT),
            referer: headers.get(REFERER),
            x_real_ip: None,
            forwarded_for: None,
        }
    }

    #[test]
    fn render_basic_format() {
        let fmt = AccessLogFormat::parse("%{method} %{uri} %{status} %{bytes}").unwrap();
        let uri = Uri::from_static("/index.html");
        let headers = HeaderMap::new();
        let ctx = LogContext {
            bytes: 1024,
            remote_addr: Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                54321,
            )),
            ..context(&uri, &headers)
        };
        assert_eq!(render(&fmt, &ctx), "GET /index.html 200 1024");
    }

    #[test]
    fn render_uri_logs_the_path_and_query_only() {
        // Over HTTP/2 the request URI carries the scheme and authority
        let fmt = AccessLogFormat::parse("%{uri}").unwrap();
        let uri = Uri::from_static("https://127.0.0.1:8443/a/b.css?v=1");
        let headers = HeaderMap::new();
        assert_eq!(render(&fmt, &context(&uri, &headers)), "/a/b.css?v=1");
    }

    #[test]
    fn render_uri_logs_the_connect_authority() {
        let fmt = AccessLogFormat::parse("%{uri}").unwrap();
        let headers = HeaderMap::new();
        let cases = [
            ("example.com:443", "example.com:443"),
            ("user:secret@example.com:443", "example.com:443"),
            ("http://example.com", "/"),
        ];
        for (uri, target) in cases {
            let uri = Uri::from_static(uri);
            assert_eq!(render(&fmt, &context(&uri, &headers)), target, "{uri}");
        }
    }

    #[test]
    fn render_missing_values_show_dash() {
        let fmt = AccessLogFormat::parse(
            "%{remote_addr} %{x_real_ip} %{forwarded_for} %{host} %{user_agent} %{referer} %{bytes}",
        )
        .unwrap();
        let uri = Uri::from_static("/");
        let headers = HeaderMap::new();
        assert_eq!(render(&fmt, &context(&uri, &headers)), "- - - - - - -");
    }

    #[test]
    fn render_percent_literal() {
        let fmt = AccessLogFormat::parse("100%% %{method}").unwrap();
        let uri = Uri::from_static("/");
        let headers = HeaderMap::new();
        assert_eq!(render(&fmt, &context(&uri, &headers)), "100% GET");
    }

    #[test]
    fn render_escapes_client_controlled_values() {
        let fmt = AccessLogFormat::parse("\"%{user_agent}\" \"%{referer}\"").unwrap();
        let uri = Uri::from_static("/");
        let mut headers = HeaderMap::new();
        headers.insert(
            hyper::header::USER_AGENT,
            "curl\" 200 \\ \tx".parse().unwrap(),
        );
        headers.insert(
            hyper::header::REFERER,
            hyper::header::HeaderValue::from_bytes(b"caf\xc3\xa9").unwrap(),
        );
        assert_eq!(
            render(&fmt, &context(&uri, &headers)),
            r#""curl\x22 200 \x5C \x09x" "caf\xC3\xA9""#
        );
    }

    #[test]
    fn render_host_prefers_the_uri_authority() {
        let fmt = AccessLogFormat::parse("%{host}").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(HOST, "other.example".parse().unwrap());
        let cases = [
            ("http://vhost.example/", "vhost.example"),
            ("https://example.com:8443/", "example.com:8443"),
            ("http://[::1]:8080/", "[::1]:8080"),
            ("http://user:secret@example.com/", "example.com"),
            ("/", "other.example"),
        ];
        for (uri, host) in cases {
            let uri = Uri::from_static(uri);
            assert_eq!(render(&fmt, &context(&uri, &headers)), host, "{uri}");
        }
    }

    #[test]
    fn push_escaped_cuts_long_values() {
        let mut buf = String::new();
        push_escaped(&mut buf, "a".repeat(MAX_ESCAPED_LEN).as_bytes());
        assert_eq!(buf, "a".repeat(MAX_ESCAPED_LEN));

        let mut buf = String::new();
        push_escaped(&mut buf, "a".repeat(MAX_ESCAPED_LEN + 1).as_bytes());
        assert_eq!(buf, format!("{}...", "a".repeat(MAX_ESCAPED_LEN)));
    }

    #[test]
    fn push_escaped_never_cuts_an_escape() {
        let mut buf = String::new();
        push_escaped(&mut buf, &[0xff; MAX_ESCAPED_LEN / 4]);
        assert_eq!(buf, "\\xFF".repeat(MAX_ESCAPED_LEN / 4));

        let mut value = "a".repeat(MAX_ESCAPED_LEN - 3).into_bytes();
        value.push(0xff);
        let mut buf = String::from("ua=");
        push_escaped(&mut buf, &value);
        assert_eq!(buf, format!("ua={}...", "a".repeat(MAX_ESCAPED_LEN - 3)));
    }

    #[test]
    fn render_all_tokens() {
        let fmt = AccessLogFormat::parse(
            "%{method}|%{uri}|%{version}|%{status}|%{bytes}|%{duration}|%{remote_addr}|\
             %{x_real_ip}|%{forwarded_for}|%{host}|%{user_agent}|%{referer}",
        )
        .unwrap();
        let uri = Uri::from_static("/a/b.css?v=1");
        let mut headers = HeaderMap::new();
        headers.insert(hyper::header::HOST, "example.com".parse().unwrap());
        headers.insert(hyper::header::USER_AGENT, "curl/8.0".parse().unwrap());
        headers.insert(
            hyper::header::REFERER,
            "https://example.com/".parse().unwrap(),
        );
        let ctx = LogContext {
            version: http_version_str(hyper::Version::HTTP_2),
            status: 304,
            bytes: 1024,
            duration_secs: 0.0012345,
            remote_addr: Some("[2001:db8::1]:443".parse().unwrap()),
            x_real_ip: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7))),
            forwarded_for: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 195))),
            ..context(&uri, &headers)
        };
        assert_eq!(
            render(&fmt, &ctx),
            "GET|/a/b.css?v=1|HTTP/2|304|1024|0.001234|2001:db8::1|\
             203.0.113.7|203.0.113.195|example.com|curl/8.0|https://example.com/"
        );
    }

    #[test]
    fn render_timestamp_uses_the_common_log_format() {
        let fmt = AccessLogFormat::parse("[%{timestamp}]").unwrap();
        let uri = Uri::from_static("/");
        let headers = HeaderMap::new();
        let line = render(&fmt, &context(&uri, &headers));
        let clf =
            regex_lite::Regex::new(r"^\[\d{2}/[A-Z][a-z]{2}/\d{4}:\d{2}:\d{2}:\d{2} [+-]\d{4}\]$")
                .unwrap();
        assert!(clf.is_match(&line), "{line}");
    }

    #[test]
    fn push_clf_time_formats_the_offset() {
        let cases = [
            (
                datetime!(2026-05-29 23:43:47 +02:00),
                "29/May/2026:23:43:47 +0200",
            ),
            (
                datetime!(2026-01-02 03:04:05 -05:30),
                "02/Jan/2026:03:04:05 -0530",
            ),
            (
                datetime!(2026-12-31 00:00:00 UTC),
                "31/Dec/2026:00:00:00 +0000",
            ),
        ];
        for (time, expected) in cases {
            let mut buf = String::new();
            push_clf_time(&mut buf, time);
            assert_eq!(buf, expected);
        }
    }

    proptest! {
        /// Escaped values never contain a byte that could end a quoted field, start a new line or
        /// form a terminal escape sequence, and they decode back to the original bytes.
        #[test]
        fn prop_push_escaped_is_printable_and_lossless(value in proptest::collection::vec(any::<u8>(), 0..64)) {
            let mut buf = String::new();
            push_escaped(&mut buf, &value);

            let mut decoded = Vec::new();
            let mut rest = buf.as_bytes();
            while let Some((&b, tail)) = rest.split_first() {
                prop_assert!((0x20..=0x7e).contains(&b) && b != b'"', "{buf}");
                if b == b'\\' {
                    prop_assert_eq!(tail.first(), Some(&b'x'), "{}", buf);
                    let hex = std::str::from_utf8(&tail[1..3]).unwrap();
                    decoded.push(u8::from_str_radix(hex, 16).unwrap());
                    rest = &tail[3..];
                } else {
                    decoded.push(b);
                    rest = tail;
                }
            }
            prop_assert_eq!(decoded, value);
        }

        /// Escaped values stop at `MAX_ESCAPED_LEN` plus `...`, after the longest prefix that fits.
        #[test]
        fn prop_push_escaped_is_bounded(value in proptest::collection::vec(any::<u8>(), 0..2048)) {
            let width = |b: u8| {
                if (0x20..=0x7e).contains(&b) && b != b'"' && b != b'\\' { 1 } else { 4 }
            };
            let mut buf = String::new();
            push_escaped(&mut buf, &value);

            let full_len: usize = value.iter().map(|&b| width(b)).sum();
            if full_len <= MAX_ESCAPED_LEN {
                prop_assert_eq!(buf.len(), full_len);
            } else {
                let body = buf.strip_suffix("...").expect("a cut value ends with `...`");
                prop_assert!(body.len() <= MAX_ESCAPED_LEN);
                let kept = body.len() - 3 * body.matches('\\').count();
                prop_assert!(body.len() + width(value[kept]) > MAX_ESCAPED_LEN);
            }
        }
    }
}
