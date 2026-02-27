// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Configurable access logging with format string support.
//!
//! Supports a `%{token}` syntax for user-defined log formats.
//! When no format is configured, falls back to the legacy request logging behavior.

use hyper::{Body, Request, Response};
use std::fmt::Write;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use crate::{handler::RequestHandlerOpts, health};

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
    pub(crate) fn parse(format: &str) -> crate::Result<Self> {
        let mut segments = Vec::new();
        let mut literal = String::new();
        let bytes = format.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            if bytes[i] == b'%' {
                if i + 1 >= len {
                    bail!("access log format: trailing '%' at end of string");
                }
                match bytes[i + 1] {
                    b'%' => {
                        literal.push('%');
                        i += 2;
                    }
                    b'{' => {
                        // Flush accumulated literal
                        if !literal.is_empty() {
                            segments.push(Segment::Literal(std::mem::take(&mut literal)));
                        }
                        i += 2; // skip '%{'
                        let start = i;
                        while i < len && bytes[i] != b'}' {
                            i += 1;
                        }
                        if i >= len {
                            bail!("access log format: unclosed '%{{' starting at position {start}");
                        }
                        let name = &format[start..i];
                        let token = Token::from_name(name).ok_or_else(|| {
                            anyhow::anyhow!("access log format: unknown token '%{{{name}}}'")
                        })?;
                        segments.push(Segment::Token(token));
                        i += 1; // skip '}'
                    }
                    _ => {
                        bail!(
                            "access log format: unexpected character '{}' after '%' at position {}; use %{{name}} for tokens or %%%% for literal %%",
                            bytes[i + 1] as char,
                            i
                        );
                    }
                }
            } else {
                literal.push(bytes[i] as char);
                i += 1;
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
                Segment::Token(token) => render_token(&mut buf, token, ctx),
            }
        }
        buf
    }
}

/// All data available for access log rendering.
struct LogContext<'a> {
    method: &'a str,
    uri: &'a str,
    version: &'a str,
    status: u16,
    bytes: u64,
    duration_secs: f64,
    remote_addr: Option<SocketAddr>,
    host: &'a str,
    user_agent: &'a str,
    referer: &'a str,
    x_real_ip: Option<IpAddr>,
    forwarded_for: Option<IpAddr>,
}

fn render_token(buf: &mut String, token: &Token, ctx: &LogContext<'_>) {
    match token {
        Token::Method => buf.push_str(ctx.method),
        Token::Uri => buf.push_str(ctx.uri),
        Token::Version => buf.push_str(ctx.version),
        Token::Status => {
            let _ = write!(buf, "{}", ctx.status);
        }
        Token::Bytes => {
            if ctx.bytes > 0 {
                let _ = write!(buf, "{}", ctx.bytes);
            } else {
                buf.push('-');
            }
        }
        Token::Duration => {
            let _ = write!(buf, "{:.6}", ctx.duration_secs);
        }
        Token::RemoteAddr => match ctx.remote_addr {
            Some(addr) => {
                let _ = write!(buf, "{}", addr.ip());
            }
            None => buf.push('-'),
        },
        Token::XRealIp => match ctx.x_real_ip {
            Some(ip) => {
                let _ = write!(buf, "{ip}");
            }
            None => buf.push('-'),
        },
        Token::ForwardedFor => match ctx.forwarded_for {
            Some(ip) => {
                let _ = write!(buf, "{ip}");
            }
            None => buf.push('-'),
        },
        Token::Host => {
            if ctx.host.is_empty() {
                buf.push('-');
            } else {
                buf.push_str(ctx.host);
            }
        }
        Token::UserAgent => {
            if ctx.user_agent.is_empty() {
                buf.push('-');
            } else {
                buf.push_str(ctx.user_agent);
            }
        }
        Token::Referer => {
            if ctx.referer.is_empty() {
                buf.push('-');
            } else {
                buf.push_str(ctx.referer);
            }
        }
        Token::Timestamp => {
            // Uses local time, respecting the TZ environment variable
            let now = chrono::Local::now();
            let _ = write!(buf, "{}", now.format("%d/%b/%Y:%H:%M:%S %z"));
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

// --- Trusted proxy helpers (absorbed from log_addr.rs) ---

fn is_trusted_proxy(opts: &RequestHandlerOpts, remote_addr: Option<SocketAddr>) -> bool {
    opts.trusted_proxies.is_empty()
        || remote_addr.is_some_and(|addr| opts.trusted_proxies.contains(&addr.ip()))
}

fn extract_x_real_ip<T>(
    req: &Request<T>,
    opts: &RequestHandlerOpts,
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

fn extract_forwarded_for<T>(
    req: &Request<T>,
    opts: &RequestHandlerOpts,
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

fn header_str(req: &Request<Body>, name: hyper::header::HeaderName) -> &str {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
}

// --- Public API ---

/// Initializes access logging.
pub(crate) fn init(
    log_remote_address: bool,
    has_custom_format: bool,
    handler_opts: &mut RequestHandlerOpts,
) {
    handler_opts.log_remote_address = log_remote_address;

    if has_custom_format {
        tracing::info!("access log: custom format enabled");
    } else {
        let trusted = if handler_opts.trusted_proxies.is_empty() {
            "all".to_owned()
        } else {
            format!("{:?}", handler_opts.trusted_proxies)
        };
        tracing::info!(
            "log requests with remote IP addresses: enabled={log_remote_address}"
        );
        tracing::info!(
            "log X-Real-IP header: enabled={}",
            handler_opts.log_x_real_ip
        );
        tracing::info!(
            "log X-Forwarded-For header: enabled={}",
            handler_opts.log_forwarded_for
        );
        tracing::info!("trusted IPs for X-Forwarded-For: {trusted}");
    }
}

/// Legacy pre-response logging (when no custom format is configured).
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
    if let Some(real_ip) = extract_x_real_ip(req, opts, remote_addr) {
        remote_addrs.push_str(format!(" x_real_ip={real_ip}").as_str());
    }
    if let Some(forwarded_for) = extract_forwarded_for(req, opts, remote_addr) {
        remote_addrs.push_str(format!(" real_remote_ip={forwarded_for}").as_str());
    }

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

/// Post-response logging using the configured access log format.
pub(crate) fn post_process(
    format: &AccessLogFormat,
    opts: &RequestHandlerOpts,
    req: &Request<Body>,
    remote_addr: Option<SocketAddr>,
    resp: &Response<Body>,
    elapsed: Duration,
) {
    // Skip custom format logging for health endpoints at debug level
    let is_health = opts.health && health::is_health_endpoint(req);

    let bytes = resp
        .headers()
        .get(hyper::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let ctx = LogContext {
        method: req.method().as_str(),
        uri: &req.uri().to_string(),
        version: http_version_str(req.version()),
        status: resp.status().as_u16(),
        bytes,
        duration_secs: elapsed.as_secs_f64(),
        remote_addr,
        host: header_str(req, hyper::header::HOST),
        user_agent: header_str(req, hyper::header::USER_AGENT),
        referer: header_str(req, hyper::header::REFERER),
        x_real_ip: extract_x_real_ip(req, opts, remote_addr),
        forwarded_for: extract_forwarded_for(req, opts, remote_addr),
    };

    let line = format.render(&ctx);

    if is_health {
        tracing::debug!("{line}");
    } else {
        tracing::info!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::RequestHandlerOpts;
    use hyper::{Body, Request, Response, StatusCode};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;

    fn make_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    fn make_request_with_headers(
        method: &str,
        uri: &str,
        headers: Vec<(&str, &str)>,
    ) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        for (k, v) in headers {
            builder = builder.header(k, v);
        }
        builder.body(Body::empty()).unwrap()
    }

    fn make_response(status: u16, content_length: u64) -> Response<Body> {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::from_u16(status).unwrap();
        if content_length > 0 {
            resp.headers_mut().insert(
                hyper::header::CONTENT_LENGTH,
                content_length.to_string().parse().unwrap(),
            );
        }
        resp
    }

    // --- Parsing tests ---

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
    fn parse_single_token() {
        let fmt = AccessLogFormat::parse("%{method}").unwrap();
        assert_eq!(fmt.segments.len(), 1);
        assert!(matches!(&fmt.segments[0], Segment::Token(Token::Method)));
    }

    #[test]
    fn parse_mixed() {
        let fmt =
            AccessLogFormat::parse("%{remote_addr} \"%{method} %{uri}\" %{status}").unwrap();
        assert_eq!(fmt.segments.len(), 7);
        assert!(matches!(&fmt.segments[0], Segment::Token(Token::RemoteAddr)));
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
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bogus"));
    }

    #[test]
    fn parse_unclosed_brace_fails() {
        let result = AccessLogFormat::parse("%{method");
        assert!(result.is_err());
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

    // --- Rendering tests ---

    #[test]
    fn render_basic_format() {
        let fmt = AccessLogFormat::parse("%{method} %{uri} %{status} %{bytes}").unwrap();
        let ctx = LogContext {
            method: "GET",
            uri: "/index.html",
            version: "HTTP/1.1",
            status: 200,
            bytes: 1024,
            duration_secs: 0.005,
            remote_addr: Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                54321,
            )),
            host: "example.com",
            user_agent: "curl/8.0",
            referer: "",
            x_real_ip: None,
            forwarded_for: None,
        };
        assert_eq!(fmt.render(&ctx), "GET /index.html 200 1024");
    }

    #[test]
    fn render_missing_values_show_dash() {
        let fmt = AccessLogFormat::parse(
            "%{remote_addr} %{x_real_ip} %{forwarded_for} %{referer} %{bytes}",
        )
        .unwrap();
        let ctx = LogContext {
            method: "GET",
            uri: "/",
            version: "HTTP/1.1",
            status: 200,
            bytes: 0,
            duration_secs: 0.0,
            remote_addr: None,
            host: "",
            user_agent: "",
            referer: "",
            x_real_ip: None,
            forwarded_for: None,
        };
        assert_eq!(fmt.render(&ctx), "- - - - -");
    }

    #[test]
    fn render_version() {
        assert_eq!(http_version_str(hyper::Version::HTTP_11), "HTTP/1.1");
        assert_eq!(http_version_str(hyper::Version::HTTP_2), "HTTP/2");
    }

    #[test]
    fn render_percent_literal() {
        let fmt = AccessLogFormat::parse("100%% %{method}").unwrap();
        let ctx = LogContext {
            method: "GET",
            uri: "/",
            version: "HTTP/1.1",
            status: 200,
            bytes: 0,
            duration_secs: 0.0,
            remote_addr: None,
            host: "",
            user_agent: "",
            referer: "",
            x_real_ip: None,
            forwarded_for: None,
        };
        assert_eq!(fmt.render(&ctx), "100% GET");
    }

    // --- IP extraction tests ---

    #[test]
    fn extract_x_real_ip_trusted() {
        let req = make_request_with_headers("GET", "/", vec![("X-Real-IP", "10.0.0.1")]);
        let opts = RequestHandlerOpts {
            log_x_real_ip: true,
            trusted_proxies: vec![],
            ..Default::default()
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        let ip = extract_x_real_ip(&req, &opts, Some(addr));
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn extract_x_real_ip_untrusted() {
        let req = make_request_with_headers("GET", "/", vec![("X-Real-IP", "10.0.0.1")]);
        let opts = RequestHandlerOpts {
            log_x_real_ip: true,
            trusted_proxies: vec![IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))],
            ..Default::default()
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 1234);
        let ip = extract_x_real_ip(&req, &opts, Some(addr));
        assert_eq!(ip, None);
    }

    #[test]
    fn extract_forwarded_for_first_ip() {
        let req = make_request_with_headers(
            "GET",
            "/",
            vec![("X-Forwarded-For", "10.0.0.1, 10.0.0.2")],
        );
        let opts = RequestHandlerOpts {
            log_forwarded_for: true,
            trusted_proxies: vec![],
            ..Default::default()
        };
        let ip = extract_forwarded_for(&req, &opts, None);
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn extract_disabled_returns_none() {
        let req = make_request_with_headers("GET", "/", vec![("X-Real-IP", "10.0.0.1")]);
        let opts = RequestHandlerOpts {
            log_x_real_ip: false,
            ..Default::default()
        };
        assert_eq!(extract_x_real_ip(&req, &opts, None), None);
    }

    // --- post_process integration test ---

    #[test]
    fn post_process_renders_log_line() {
        let fmt = AccessLogFormat::parse("%{method} %{uri} %{status}").unwrap();
        let opts = RequestHandlerOpts::default();
        let req = make_request("GET", "/test");
        let resp = make_response(200, 512);
        // Just verify it doesn't panic; actual log output goes through tracing
        post_process(
            &fmt,
            &opts,
            &req,
            None,
            &resp,
            Duration::from_millis(5),
        );
    }
}
