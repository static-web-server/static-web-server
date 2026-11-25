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
    let vhosts = vhosts_opts?;
    if vhosts.is_empty() {
        return None;
    }

    let request_host_str = request_host(req)?;

    for vhost in vhosts {
        if vhost.host == request_host_str {
            // PERF/LOGGING: This fires on every matched request (one per
            // HTTP exchange that targets a configured vhost), so it must
            // not run at `info` level \u2014 it would amplify access-log
            // volume by an order of magnitude and risk leaking request
            // URIs into operator-facing logs. Use `debug` instead.
            tracing::debug!(
                "virtual host matched: vhost={} vhost_root={} method={} uri={}",
                vhost.host,
                vhost.root.display(),
                req.method(),
                req.uri(),
            );
            return Some(&vhost.root);
        }
    }
    None
}

/// Returns the request host without its port: the HTTP/2 `:authority`, otherwise the
/// `Host` header.
///
/// URL rules read the host the same way, so a request can't reach a virtual host while
/// skipping the rules bound to it.
pub(crate) fn request_host<T>(req: &Request<T>) -> Option<&str> {
    RequestHost::from_request(req).map(|host| host.name())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Request, Uri, header::HOST};

    fn create_vhost(host: &str, root: &str) -> VirtualHosts {
        VirtualHosts {
            host: host.to_string(),
            root: PathBuf::from(root),
        }
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
