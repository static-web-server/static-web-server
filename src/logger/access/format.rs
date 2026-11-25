// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The compiled access log format and its `%{token}` parser.
//!

use std::sync::Arc;

/// A compiled access log format, parsed once at startup.
#[derive(Debug, Clone)]
pub struct AccessLogFormat {
    pub(crate) segments: Arc<[Segment]>,
}

/// A single segment of a compiled format string.
#[derive(Debug)]
pub(crate) enum Segment {
    Literal(String),
    Token(Token),
}

/// Available log format tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Token {
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

        Ok(Self {
            segments: segments.into(),
        })
    }

    /// Whether the format logs `X-Real-IP` or `X-Forwarded-For`.
    pub(crate) fn uses_proxy_headers(&self) -> bool {
        self.segments
            .iter()
            .any(|seg| matches!(seg, Segment::Token(Token::XRealIp | Token::ForwardedFor)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn uses_proxy_headers_only_with_proxy_tokens() {
        let uses = |f: &str| AccessLogFormat::parse(f).unwrap().uses_proxy_headers();
        assert!(uses("%{x_real_ip}"));
        assert!(uses("%{method} %{forwarded_for}"));
        assert!(!uses("%{remote_addr} x_real_ip forwarded_for"));
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
}
