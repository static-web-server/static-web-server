// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that allows to determine a virtual hostname.
//!

use hyper::Request;
use std::path::PathBuf;

use crate::settings::VirtualHosts;
use crate::url_rules::RequestHost;

/// It returns different root directory if the "Host" header matches a virtual hostname.
pub(crate) fn get_real_root<'a, T>(
    req: &mut Request<T>,
    vhosts_opts: Option<&'a [VirtualHosts]>,
) -> Option<&'a PathBuf> {
    let vhost = find(req, vhosts_opts?)?;
    // PERF/LOGGING: This fires on every matched request (one per
    // HTTP exchange that targets a configured vhost), so it must
    // not run at `info` level \u2014 it would amplify access-log
    // volume by an order of magnitude and risk leaking request
    // URIs into operator-facing logs. Use `debug` instead.
    tracing::debug!(
        "virtual host matched: vhost={} vhost_root={} method={} uri={}",
        vhost.name,
        vhost.root.display(),
        req.method(),
        req.uri(),
    );
    Some(&vhost.root)
}

/// Returns the first of `vhosts` that `req` is addressed to.
///
/// The request host is read and compared as for URL rules: the HTTP/2 `:authority`,
/// otherwise the `Host` header, in any letter case and IPv6 notation. So a request that a
/// rule bound to a host applies to is served from the virtual host of that host.
pub(crate) fn find<'a, T>(
    req: &Request<T>,
    vhosts: &'a [VirtualHosts],
) -> Option<&'a VirtualHosts> {
    let host = RequestHost::from_request(req)?;
    vhosts.iter().find(|vhost| vhost.host.matches(&host))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url_rules::RuleHost;
    use hyper::{Request, Uri, header::HOST};

    fn create_vhost(host: &str, root: &str) -> VirtualHosts {
        VirtualHosts {
            host: RuleHost::parse(host).unwrap(),
            name: host.to_string(),
            root: PathBuf::from(root),
        }
    }

    fn real_root(vhosts: &[VirtualHosts], uri: &str, host: Option<&str>) -> Option<PathBuf> {
        let mut builder = Request::builder().uri(uri);
        if let Some(host) = host {
            builder = builder.header(HOST, host);
        }
        let mut req = builder.body(crate::body::empty()).unwrap();
        get_real_root(&mut req, Some(vhosts)).cloned()
    }

    #[test]
    fn test_get_real_root_match_any_letter_case() {
        let vhosts = [create_vhost("Example.com", "/var/www/example")];
        for host in ["example.com", "EXAMPLE.COM", "eXample.Com:8080"] {
            assert_eq!(
                real_root(&vhosts, "/", Some(host)),
                Some("/var/www/example".into()),
                "{host}"
            );
        }
        assert_eq!(
            real_root(&vhosts, "http://EXAMPLE.com/", None),
            Some("/var/www/example".into())
        );
    }

    #[test]
    fn test_get_real_root_match_ipv6_in_any_notation() {
        let vhosts = [create_vhost("[2001:db8::1]", "/var/www/v6")];
        for host in ["[2001:db8::1]", "[2001:DB8:0:0::1]:8080"] {
            assert_eq!(
                real_root(&vhosts, "/", Some(host)),
                Some("/var/www/v6".into()),
                "{host}"
            );
        }
        assert_eq!(real_root(&vhosts, "/", Some("[2001:db8::2]")), None);
    }

    #[test]
    fn test_get_real_root_with_port_requires_that_port() {
        let vhosts = [create_vhost("example.com:8080", "/var/www/example")];
        assert_eq!(
            real_root(&vhosts, "/", Some("example.com:8080")),
            Some("/var/www/example".into())
        );
        for host in ["example.com", "example.com:9090"] {
            assert_eq!(real_root(&vhosts, "/", Some(host)), None, "{host}");
        }
    }

    #[test]
    fn test_get_real_root_first_match_wins() {
        let vhosts = [
            create_vhost("example.com", "/var/www/first"),
            create_vhost("EXAMPLE.COM", "/var/www/second"),
        ];
        assert_eq!(
            real_root(&vhosts, "/", Some("Example.com")),
            Some("/var/www/first".into())
        );
    }

    #[test]
    fn test_get_real_root_match_http1() {
        let vhosts = [
            create_vhost("example.com", "/var/www/example"),
            create_vhost("test.com", "/var/www/test"),
        ];
        let mut req = Request::builder()
            .uri("http://example.com/")
            .header(HOST, "example.com")
            .body(crate::body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&vhosts));
        assert_eq!(result, Some(&PathBuf::from("/var/www/example")));
    }

    #[test]
    fn test_get_real_root_match_http1_with_port() {
        let vhosts = [create_vhost("example.com", "/var/www/example")];
        let mut req = Request::builder()
            .uri("http://example.com:8080/")
            .header(HOST, "example.com:8080")
            .body(crate::body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&vhosts));
        assert_eq!(result, Some(&PathBuf::from("/var/www/example")));
    }

    #[test]
    fn test_get_real_root_match_http2_authority() {
        let vhosts = [create_vhost("example.com", "/var/www/example")];
        let mut req = Request::builder()
            .uri(Uri::builder().authority("example.com").build().unwrap())
            .body(crate::body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&vhosts));
        assert_eq!(result, Some(&PathBuf::from("/var/www/example")));
    }

    #[test]
    fn test_get_real_root_no_match() {
        let vhosts = [create_vhost("example.com", "/var/www/example")];
        let mut req = Request::builder()
            .uri("http://example2.com/")
            .header(HOST, "example2.com")
            .body(crate::body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&vhosts));
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_real_root_port_forms() {
        let vhosts = [create_vhost("example.com", "/var/www/example")];
        let real_root = |host: &str| {
            let mut req = Request::builder()
                .uri("/")
                .header(HOST, host)
                .body(crate::body::empty())
                .unwrap();
            get_real_root(&mut req, Some(&vhosts)).cloned()
        };
        for host in ["example.com:80", "example.com:080", "example.com:"] {
            assert_eq!(real_root(host), Some("/var/www/example".into()), "{host}");
        }
        for host in ["example.com:+80", "example.com:8a", "example.com:65536"] {
            assert_eq!(real_root(host), None, "{host}");
        }
    }

    #[test]
    fn test_get_real_root_no_vhosts() {
        let mut req = Request::builder()
            .uri("http://example.com/")
            .header(HOST, "example.com")
            .body(crate::body::empty())
            .unwrap();

        let result = get_real_root(&mut req, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_real_root_empty_vhosts() {
        let mut req = Request::builder()
            .uri("http://example.com/")
            .header(HOST, "example.com")
            .body(crate::body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&[]));
        assert_eq!(result, None);
    }
}
