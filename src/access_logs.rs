// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Configurable access logging with format string support.
//!
//! Supports a `%{token}` syntax for user-defined log formats.
//! When no format is configured, `log_addr` logs requests as before.
//!
//! Values taken from the request (URI, `Host`, `User-Agent`, `Referer`) are client-controlled.
//! Bytes outside printable ASCII, `"` and `\` in them are written as `\xHH`, so a client cannot
//! end a quoted field, start a new log line or inject terminal escape sequences. An escaped
//! value longer than 4 KiB is cut and ends with `...`, so a large header cannot make a large line.

use hyper::header::{HOST, HeaderName, HeaderValue};
use hyper::{HeaderMap, Request, Response, Uri};
use std::fmt::Write;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use crate::{body::Body, handler::RequestHandlerOpts, health, log_addr};

/// A compiled access log format, parsed once at startup.
#[derive(Debug)]
pub struct AccessLogFormat {
    segments: Vec<Segment>,
}

/// A single segment of a compiled format string.
#[derive(Debug)]
enum Segment {
    Literal(String),
    Token(Token),
}

/// Available log format tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Method,
    Uri,
    Status,
    Bytes,
    Duration,
    RemoteAddr,
    XRealIp,
    ForwardedFor,
    Host,
    UserAgent,
    Referer,
    Timestamp,
    Version,
}

impl Token {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "method" => Some(Self::Method),
            "uri" => Some(Self::Uri),
            "status" => Some(Self::Status),
            "bytes" => Some(Self::Bytes),
            "duration" => Some(Self::Duration),
            "remote_addr" => Some(Self::RemoteAddr),
            "x_real_ip" => Some(Self::XRealIp),
            "forwarded_for" => Some(Self::ForwardedFor),
            "host" => Some(Self::Host),
            "user_agent" => Some(Self::UserAgent),
            "referer" => Some(Self::Referer),
            "timestamp" => Some(Self::Timestamp),
            "version" => Some(Self::Version),
            _ => None,
        }
    }
}

impl AccessLogFormat {
    /// Parses a format string into a compiled representation.
    ///
    /// Tokens use `%{name}` syntax. Use `%%` for a literal `%`.
    pub fn parse(format: &str) -> crate::Result<Self> {
        let mut segments = Vec::new();
        let mut literal = String::new();
        let mut chars = format.chars();

        while let Some(c) = chars.next() {
            if c != '%' {
                literal.push(c);
                continue;
            }
            match chars.next() {
                Some('%') => literal.push('%'),
                Some('{') => {
                    let mut name = String::new();
                    let mut closed = false;
                    for ch in chars.by_ref() {
                        if ch == '}' {
                            closed = true;
                            break;
                        }
                        name.push(ch);
                    }
                    if !closed {
                        bail!("access log format: unclosed '%{{{name}'");
                    }
                    let token = Token::from_name(&name)
                        .ok_or_else(|| anyhow!("access log format: unknown token '%{{{name}}}'"))?;
                    if !literal.is_empty() {
                        segments.push(Segment::Literal(std::mem::take(&mut literal)));
                    }
                    segments.push(Segment::Token(token));
                }
                Some(other) => bail!(
                    "access log format: unexpected character '{other}' after '%'; \
                     use %{{name}} for tokens or %% for a literal %"
                ),
                None => bail!("access log format: trailing '%' at end of string"),
            }
        }

        if !literal.is_empty() {
            segments.push(Segment::Literal(literal));
        }

        Ok(Self { segments })
    }

    /// Renders the format using the given request/response context.
    fn render(&self, ctx: &LogContext<'_>) -> String {
        let mut buf = String::with_capacity(256);
        for seg in &self.segments {
            match seg {
                Segment::Literal(s) => buf.push_str(s),
                Segment::Token(token) => render_token(&mut buf, *token, ctx),
            }
        }
        buf
    }
}

/// The request values that the pipeline may replace, as the client sent them.
#[derive(Debug)]
pub(crate) struct ClientRequest {
    uri: Uri,
    host: Option<HeaderValue>,
}

impl ClientRequest {
    /// Keeps the URI and `Host` header, which rewrites replace, before the pipeline runs.
    pub(crate) fn new<T>(req: &Request<T>) -> Self {
        Self {
            uri: req.uri().clone(),
            host: req.headers().get(HOST).cloned(),
        }
    }
}

/// All data available for access log rendering.
struct LogContext<'a> {
    method: &'a str,
    uri: &'a Uri,
    host: Option<&'a HeaderValue>,
    version: &'a str,
    status: u16,
    bytes: u64,
    duration_secs: f64,
    remote_addr: Option<SocketAddr>,
    headers: &'a HeaderMap,
    x_real_ip: Option<IpAddr>,
    forwarded_for: Option<IpAddr>,
}

fn render_token(buf: &mut String, token: Token, ctx: &LogContext<'_>) {
    match token {
        Token::Method => buf.push_str(ctx.method),
        Token::Uri => {
            let path = ctx.uri.path_and_query().map_or("/", |p| p.as_str());
            push_escaped(buf, path.as_bytes());
        }
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
        Token::UserAgent => push_header(buf, ctx.headers, hyper::header::USER_AGENT),
        Token::Referer => push_header(buf, ctx.headers, hyper::header::REFERER),
        Token::Timestamp => {
            // Uses local time, respecting the TZ environment variable
            let now = chrono::Local::now();
            let _ = write!(buf, "{}", now.format("%d/%b/%Y:%H:%M:%S %z"));
        }
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
        Some(authority) => {
            let authority = authority.as_str();
            let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
            Some(host_port.as_bytes())
        }
        None => host.map(HeaderValue::as_bytes),
    }
}

fn push_header(buf: &mut String, headers: &HeaderMap, name: HeaderName) {
    push_value(buf, headers.get(name).map(HeaderValue::as_bytes));
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

fn http_version_str(version: hyper::Version) -> &'static str {
    match version {
        hyper::Version::HTTP_09 => "HTTP/0.9",
        hyper::Version::HTTP_10 => "HTTP/1.0",
        hyper::Version::HTTP_11 => "HTTP/1.1",
        hyper::Version::HTTP_2 => "HTTP/2",
        hyper::Version::HTTP_3 => "HTTP/3",
        _ => "HTTP/?",
    }
}

/// Logs the request with the configured access log format, once the response exists.
///
/// It only logs, unlike the `post_process` steps that take and return the response. `client`
/// holds the URI and `Host` the client sent: rewrites replace them during the pipeline.
pub(crate) fn log_response<T>(
    format: &AccessLogFormat,
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    client: &ClientRequest,
    remote_addr: Option<SocketAddr>,
    resp: &Response<Body>,
    elapsed: Duration,
) {
    let bytes = resp
        .headers()
        .get(hyper::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or_default();

    let ctx = LogContext {
        method: req.method().as_str(),
        uri: &client.uri,
        host: client.host.as_ref(),
        version: http_version_str(req.version()),
        status: resp.status().as_u16(),
        bytes,
        duration_secs: elapsed.as_secs_f64(),
        remote_addr,
        headers: req.headers(),
        x_real_ip: log_addr::x_real_ip(opts, req, remote_addr),
        forwarded_for: log_addr::forwarded_for(opts, req, remote_addr),
    };

    let line = format.render(&ctx);

    // Log health check requests in debug mode only, as `log_addr` does
    if opts.health && health::is_health_endpoint(&client.uri) {
        tracing::debug!("{line}");
    } else {
        tracing::info!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Method;
    use proptest::prelude::*;
    use std::net::Ipv4Addr;

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
            headers,
            x_real_ip: None,
            forwarded_for: None,
        }
    }

    // Parsing tests

    #[test]
    fn parse_empty() {
        let fmt = AccessLogFormat::parse("").unwrap();
        assert!(fmt.segments.is_empty());
    }

    #[test]
    fn parse_literal_only() {
        let fmt = AccessLogFormat::parse("hello world").unwrap();
        assert_eq!(fmt.segments.len(), 1);
        assert!(matches!(&fmt.segments[0], Segment::Literal(s) if s == "hello world"));
    }

    #[test]
    fn parse_keeps_non_ascii_literals() {
        let fmt = AccessLogFormat::parse("→ %{status} ✓").unwrap();
        assert!(matches!(&fmt.segments[0], Segment::Literal(s) if s == "→ "));
        assert!(matches!(&fmt.segments[2], Segment::Literal(s) if s == " ✓"));
    }

    #[test]
    fn parse_single_token() {
        let fmt = AccessLogFormat::parse("%{method}").unwrap();
        assert_eq!(fmt.segments.len(), 1);
        assert!(matches!(&fmt.segments[0], Segment::Token(Token::Method)));
    }

    #[test]
    fn parse_mixed() {
        let fmt = AccessLogFormat::parse("%{remote_addr} \"%{method} %{uri}\" %{status}").unwrap();
        assert_eq!(fmt.segments.len(), 7);
        assert!(matches!(
            &fmt.segments[0],
            Segment::Token(Token::RemoteAddr)
        ));
        assert!(matches!(&fmt.segments[1], Segment::Literal(s) if s == " \""));
        assert!(matches!(&fmt.segments[2], Segment::Token(Token::Method)));
        assert!(matches!(&fmt.segments[3], Segment::Literal(s) if s == " "));
        assert!(matches!(&fmt.segments[4], Segment::Token(Token::Uri)));
        assert!(matches!(&fmt.segments[5], Segment::Literal(s) if s == "\" "));
        assert!(matches!(&fmt.segments[6], Segment::Token(Token::Status)));
    }

    #[test]
    fn parse_percent_escape() {
        let fmt = AccessLogFormat::parse("100%% done").unwrap();
        assert_eq!(fmt.segments.len(), 1);
        assert!(matches!(&fmt.segments[0], Segment::Literal(s) if s == "100% done"));
    }

    #[test]
    fn parse_all_tokens() {
        let all = "%{method} %{uri} %{status} %{bytes} %{duration} %{remote_addr} \
                   %{x_real_ip} %{forwarded_for} %{host} %{user_agent} %{referer} \
                   %{timestamp} %{version}";
        let fmt = AccessLogFormat::parse(all).unwrap();
        let token_count = fmt
            .segments
            .iter()
            .filter(|s| matches!(s, Segment::Token(_)))
            .count();
        assert_eq!(token_count, 13);
    }

    #[test]
    fn parse_unknown_token_fails() {
        let result = AccessLogFormat::parse("%{bogus}");
        assert!(result.unwrap_err().to_string().contains("bogus"));
    }

    #[test]
    fn parse_empty_token_fails() {
        let result = AccessLogFormat::parse("%{}");
        assert!(result.unwrap_err().to_string().contains("unknown token"));
    }

    #[test]
    fn parse_unclosed_brace_fails() {
        let result = AccessLogFormat::parse("%{method");
        assert!(result.unwrap_err().to_string().contains("unclosed"));
    }

    #[test]
    fn parse_unclosed_brace_after_a_closed_one_fails() {
        let result = AccessLogFormat::parse("%{method} %{method");
        assert!(result.unwrap_err().to_string().contains("unclosed"));
    }

    #[test]
    fn parse_trailing_percent_fails() {
        let result = AccessLogFormat::parse("hello%");
        assert!(result.is_err());
    }

    #[test]
    fn parse_bare_percent_char_fails() {
        let result = AccessLogFormat::parse("%x");
        assert!(result.is_err());
    }

    // Rendering tests

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
        assert_eq!(fmt.render(&ctx), "GET /index.html 200 1024");
    }

    #[test]
    fn render_uri_logs_the_path_and_query_only() {
        // Over HTTP/2 the request URI carries the scheme and authority
        let fmt = AccessLogFormat::parse("%{uri}").unwrap();
        let uri = Uri::from_static("https://127.0.0.1:8443/a/b.css?v=1");
        let headers = HeaderMap::new();
        assert_eq!(fmt.render(&context(&uri, &headers)), "/a/b.css?v=1");
    }

    #[test]
    fn render_missing_values_show_dash() {
        let fmt = AccessLogFormat::parse(
            "%{remote_addr} %{x_real_ip} %{forwarded_for} %{host} %{user_agent} %{referer} %{bytes}",
        )
        .unwrap();
        let uri = Uri::from_static("/");
        let headers = HeaderMap::new();
        assert_eq!(fmt.render(&context(&uri, &headers)), "- - - - - - -");
    }

    #[test]
    fn render_percent_literal() {
        let fmt = AccessLogFormat::parse("100%% %{method}").unwrap();
        let uri = Uri::from_static("/");
        let headers = HeaderMap::new();
        assert_eq!(fmt.render(&context(&uri, &headers)), "100% GET");
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
            fmt.render(&context(&uri, &headers)),
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
            assert_eq!(fmt.render(&context(&uri, &headers)), host, "{uri}");
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
            fmt.render(&ctx),
            "GET|/a/b.css?v=1|HTTP/2|304|1024|0.001234|2001:db8::1|\
             203.0.113.7|203.0.113.195|example.com|curl/8.0|https://example.com/"
        );
    }

    #[test]
    fn render_timestamp_uses_the_common_log_format() {
        let fmt = AccessLogFormat::parse("[%{timestamp}]").unwrap();
        let uri = Uri::from_static("/");
        let headers = HeaderMap::new();
        let line = fmt.render(&context(&uri, &headers));
        // e.g. `[29/May/2026:23:43:47 +0200]`
        let parsed =
            chrono::DateTime::parse_from_str(line.trim_matches(['[', ']']), "%d/%b/%Y:%H:%M:%S %z");
        assert!(parsed.is_ok(), "{line}");
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
