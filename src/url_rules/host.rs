// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Host matching for URL rules and virtual hosts.
//!
//! A rule's `host` is compared against the request host case-insensitively
//! (RFC 3986 §3.2.2), and IPv6 addresses by value. A `host` without a port matches any
//! port; a `host` with a port matches only requests carrying that same explicit port.
//! The scheme's default port is not implied: clients omit it, so `example.com:80` never
//! matches `Host: example.com`, and startup warns about it. A virtual host's `host` is
//! compared the same way, so a request a host-bound rule applies to is served from the
//! virtual host of that host.
//!
//! The client chooses the host and port it sends, whatever address it connected to. A
//! rule bound to a host only applies to content that is served for that host alone, such
//! as the root of its virtual host: the same path reached under another host skips it.

use hyper::{Request, Uri, header::HOST, http::HeaderValue};
use std::net::Ipv6Addr;

use crate::Result;

/// The `host` of a URL rule or virtual host, validated and normalized at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleHost {
    /// Host name or IPv6 address.
    name: HostName,
    /// Port the request must carry, if any.
    port: Option<u16>,
}

/// A host name or IPv4 address in lowercase, or an IPv6 address.
#[derive(Debug, Clone, PartialEq, Eq)]
enum HostName {
    /// Lowercase registered name or IPv4 address.
    Name(String),
    /// Address of a bracketed IPv6 literal.
    Ipv6(Ipv6Addr),
}

impl RuleHost {
    /// Parses a `host` value of the form `name[:port]`, where `name` is a host name,
    /// an IPv4 address, or a bracketed IPv6 address.
    ///
    /// Fails on wildcards, user info, characters outside a host name, an invalid IPv6
    /// address, and a port outside `1..=65535`.
    pub fn parse(value: &str) -> Result<Self> {
        if value.contains('*') {
            bail!("wildcards are not supported in host {value:?}");
        }
        let Some(host) = RequestHost::parse(value) else {
            bail!("invalid host {value:?}, expected `name` or `name:port`");
        };
        let Some(name) = HostName::parse(host.name) else {
            bail!("invalid host name in {value:?}");
        };
        if host.port == Some(0) {
            bail!("invalid host port 0 in {value:?}");
        }
        if let Some(port @ (80 | 443)) = host.port {
            tracing::warn!(
                "host {value:?} names the default port {port}, which clients usually omit \
                 from `Host`; leave the port out to match those requests"
            );
        }
        Ok(Self {
            name,
            port: host.port,
        })
    }

    /// Returns whether the request host is this host.
    #[must_use]
    pub(crate) fn matches(&self, host: &RequestHost<'_>) -> bool {
        self.port.is_none_or(|p| host.port == Some(p)) && self.name.matches(host.name)
    }
}

impl HostName {
    /// Parses a host without port; `None` if it is not a valid name or IPv6 literal.
    fn parse(name: &str) -> Option<Self> {
        if let Some(addr) = ipv6_literal(name) {
            return addr.map(Self::Ipv6);
        }
        // RFC 3986 §3.2.2 registered names as used in DNS, which also covers IPv4
        let is_name = name.bytes().any(|b| b.is_ascii_alphanumeric())
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_'));
        is_name.then(|| Self::Name(name.to_ascii_lowercase()))
    }

    /// Returns whether the request host name is this name, ignoring case and IPv6
    /// notation differences.
    fn matches(&self, name: &str) -> bool {
        match self {
            Self::Name(own) => own.eq_ignore_ascii_case(name),
            Self::Ipv6(own) => ipv6_literal(name).flatten() == Some(*own),
        }
    }
}

/// Returns `Some` for a bracketed name, holding its address if it is a valid IPv6 one.
fn ipv6_literal(name: &str) -> Option<Option<Ipv6Addr>> {
    let inner = name.strip_prefix('[')?;
    Some(inner.strip_suffix(']').and_then(|addr| addr.parse().ok()))
}

/// The host a request is addressed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RequestHost<'a> {
    /// Host name or address as sent by the client.
    name: &'a str,
    /// Explicit port, if the client sent one.
    port: Option<u16>,
}

impl<'a> RequestHost<'a> {
    /// Returns the host of `req`: the authority of an absolute-form or HTTP/2 request
    /// URI, otherwise the `Host` header.
    ///
    /// RFC 9112 §3.2.2: the URI authority takes precedence over `Host`. Its user info is
    /// ignored, as virtual hosts do, so it can't skip the rules of the host it reaches. A
    /// missing or malformed `Host` yields `None`, which matches no host-specific rule.
    pub(crate) fn from_request<T>(req: &'a Request<T>) -> Option<Self> {
        Self::new(req.uri(), req.headers().get(HOST))
    }

    /// Returns the host of a request with the given `uri` and `Host` header, as
    /// [`Self::from_request`] does.
    pub(crate) fn new(uri: &'a Uri, host: Option<&'a HeaderValue>) -> Option<Self> {
        if let Some(authority) = uri.authority() {
            return Some(Self {
                name: authority.host(),
                port: authority.port_u16(),
            });
        }
        Self::parse(host?.to_str().ok()?)
    }

    /// Parses `host[:port]`, where `host` may be a bracketed IPv6 address.
    ///
    /// RFC 3986 §3.2.3: an empty port is the same as no port. RFC 9110 §7.2: `Host`
    /// carries no user info, so a value with one yields `None`.
    fn parse(value: &'a str) -> Option<Self> {
        if value.contains('@') {
            return None;
        }
        let (name, port) = match value.rsplit_once(':') {
            Some((name, port)) if !port.contains(']') => (name, port),
            _ => (value, ""),
        };
        if name.is_empty() {
            return None;
        }
        let port = match port {
            "" => None,
            port if port.bytes().all(|b| b.is_ascii_digit()) => Some(port.parse().ok()?),
            _ => return None,
        };
        Some(Self { name, port })
    }
}

#[cfg(test)]
mod tests {
    use super::{HostName, RequestHost, RuleHost};
    use hyper::Request;
    use proptest::prelude::*;

    fn request(uri: &str, host: Option<&str>) -> Request<()> {
        let mut builder = Request::builder().uri(uri);
        if let Some(host) = host {
            builder = builder.header("host", host);
        }
        builder.body(()).unwrap()
    }

    fn matches(rule: &str, uri: &str, host: Option<&str>) -> bool {
        let rule = RuleHost::parse(rule).unwrap();
        let req = request(uri, host);
        RequestHost::from_request(&req).is_some_and(|host| rule.matches(&host))
    }

    #[test]
    fn rule_host_lowercases_name() {
        let host = RuleHost::parse("Example.COM:8080").unwrap();
        assert_eq!(host.name, HostName::Name("example.com".into()));
        assert_eq!(host.port, Some(8080));
    }

    #[test]
    fn rule_host_accepts_ip_addresses() {
        assert_eq!(RuleHost::parse("127.0.0.1:1234").unwrap().port, Some(1234));
        assert_eq!(
            RuleHost::parse("[::1]").unwrap().name,
            HostName::Ipv6(std::net::Ipv6Addr::LOCALHOST)
        );
        assert_eq!(RuleHost::parse("[::1]:80").unwrap().port, Some(80));
    }

    #[test]
    fn rule_host_empty_port_is_no_port() {
        assert_eq!(RuleHost::parse("example.com:").unwrap().port, None);
    }

    #[test]
    fn rule_host_rejects_invalid_values() {
        for value in [
            "",
            ":80",
            "::1",
            "user@example.com",
            "http://example.com",
            "example.com/path",
            "example.com:abc",
            "example.com:65536",
            "exa mple.com",
            "*.example.com",
            "example.com:0",
            "[zz]",
            "[]",
            "[::1]junk",
            "[fe80::1%25eth0]",
            "exa[mple]",
            "a,b",
            "(x)",
            ".",
            "-",
        ] {
            assert!(
                RuleHost::parse(value).is_err(),
                "{value:?} must be rejected"
            );
        }
    }

    #[test]
    fn host_header_matches_case_insensitively() {
        assert!(matches("example.com", "/", Some("EXAMPLE.com")));
        assert!(matches("EXAMPLE.com", "/", Some("example.com")));
    }

    #[test]
    fn host_without_port_matches_any_port() {
        assert!(matches("example.com", "/", Some("example.com")));
        assert!(matches("example.com", "/", Some("example.com:8080")));
        assert!(matches("example.com", "/", Some("example.com:")));
    }

    #[test]
    fn host_with_port_requires_same_port() {
        assert!(matches("example.com:8080", "/", Some("example.com:8080")));
        assert!(!matches("example.com:8080", "/", Some("example.com:9090")));
        assert!(!matches("example.com:8080", "/", Some("example.com")));
    }

    #[test]
    fn host_mismatch_does_not_match() {
        assert!(!matches("example.com", "/", Some("example.org")));
        assert!(!matches("example.com", "/", Some("sub.example.com")));
        assert!(!matches("example.com", "/", Some("example.com.evil")));
    }

    #[test]
    fn host_ipv6_matches_with_and_without_port() {
        assert!(matches("[::1]", "/", Some("[::1]:8080")));
        assert!(matches("[::1]:8080", "/", Some("[::1]:8080")));
        assert!(!matches("[::1]:8080", "/", Some("[::1]")));
        assert!(!matches("[::1]", "/", Some("[::2]")));
    }

    #[test]
    fn host_ipv6_matches_any_notation() {
        assert!(matches("[::1]", "/", Some("[0:0:0:0:0:0:0:1]")));
        assert!(matches("[2001:DB8::1]", "/", Some("[2001:db8:0::1]:443")));
        assert!(!matches("[::1]", "/", Some("[::1")));
    }

    #[test]
    fn rule_host_wildcard_has_its_own_error() {
        let err = RuleHost::parse("*.example.com").unwrap_err();
        assert!(
            err.to_string().contains("wildcards are not supported"),
            "{err}"
        );
    }

    #[test]
    fn host_missing_or_malformed_matches_nothing() {
        assert!(!matches("example.com", "/", None));
        assert!(!matches("example.com", "/", Some("")));
        assert!(!matches("example.com", "/", Some("example.com:abc")));
        assert!(!matches("example.com", "/", Some("example.com:65536")));
        assert!(!matches("example.com", "/", Some("user@example.com")));
    }

    #[test]
    fn host_not_utf8_matches_nothing() {
        let mut req = request("/", None);
        let host = hyper::http::HeaderValue::from_bytes(b"ex\xffample.com").unwrap();
        req.headers_mut().insert("host", host);
        assert!(RequestHost::from_request(&req).is_none());
    }

    #[test]
    fn host_uri_authority_takes_precedence_over_header() {
        assert!(matches(
            "example.com",
            "http://example.com/",
            Some("other.org")
        ));
        assert!(!matches(
            "other.org",
            "http://example.com/",
            Some("other.org")
        ));
    }

    #[test]
    fn host_uri_authority_ignores_user_info() {
        assert!(matches("example.com", "http://user@example.com/", None));
        assert!(matches("example.com:81", "http://@example.com:81/", None));
    }

    proptest! {
        /// Request host parsing takes untrusted input: it must never panic, and any
        /// port it returns must be one the client spelled out.
        #[test]
        fn prop_request_host_parse_never_panics(value in "[a-zA-Z0-9.:@\\[\\]%-]{0,24}") {
            if let Some(host) = RequestHost::parse(&value) {
                prop_assert!(!host.name.is_empty());
                prop_assert!(!value.contains('@'));
                if let Some(port) = host.port {
                    prop_assert!(value.ends_with(&port.to_string()));
                }
            }
        }

        /// A valid rule host matches a request that sends the same value, in any case.
        #[test]
        fn prop_rule_host_matches_itself(
            name in "[a-z0-9][a-z0-9.-]{0,16}",
            port in proptest::option::of(1u16..),
            upper in any::<bool>(),
        ) {
            let value = match port {
                Some(port) => format!("{name}:{port}"),
                None => name,
            };
            let rule = RuleHost::parse(&value).unwrap();
            let sent = if upper { value.to_ascii_uppercase() } else { value.clone() };
            let host = RequestHost::parse(&sent).unwrap();
            prop_assert!(rule.matches(&host), "{value} must match {sent}");
        }
    }
}
