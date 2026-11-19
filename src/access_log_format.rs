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
