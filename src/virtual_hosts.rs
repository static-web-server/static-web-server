// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that allows to determine a virtual hostname.
//!

use hyper::Request;
use hyper::header::HOST;
use std::path::PathBuf;

use crate::settings::VirtualHosts;

/// It returns different root directory if the "Host" header matches a virtual hostname.
pub(crate) fn get_real_root<'a, T>(
    req: &mut Request<T>,
    vhosts_opts: Option<&'a [VirtualHosts]>,
) -> Option<&'a PathBuf> {
    let vhosts = vhosts_opts?;
    if vhosts.is_empty() {
        return None;
    }

    let request_host_str = if let Some(authority) = req.uri().authority() {
        // HTTP2
        authority.host()
    } else {
        // HTTP1 - fall back to host header
        let host_header = req.headers().get(HOST)?.to_str().ok()?;

        // host header can include the port -> remove it
        host_header
            .rsplit_once(":")
            .and_then(|(potential_host, potential_port)| {
                potential_port
                    .parse::<u16>()
                    .is_ok()
                    .then_some(potential_host)
            })
            .unwrap_or(host_header)
    };

    for vhost in vhosts {
        if vhost.host == request_host_str {
            tracing::info!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Request, Uri};

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
            .body(Body::empty())
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
            .body(Body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&vhosts));
        assert_eq!(result, Some(&PathBuf::from("/var/www/example")));
    }

    #[test]
    fn test_get_real_root_match_http2_authority() {
        let vhosts = [create_vhost("example.com", "/var/www/example")];
        let mut req = Request::builder()
            .uri(Uri::builder().authority("example.com").build().unwrap())
            .body(Body::empty())
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
            .body(Body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&vhosts));
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_real_root_no_vhosts() {
        let mut req = Request::builder()
            .uri("http://example.com/")
            .header(HOST, "example.com")
            .body(Body::empty())
            .unwrap();

        let result = get_real_root(&mut req, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_real_root_empty_vhosts() {
        let mut req = Request::builder()
            .uri("http://example.com/")
            .header(HOST, "example.com")
            .body(Body::empty())
            .unwrap();

        let result = get_real_root(&mut req, Some(&[]));
        assert_eq!(result, None);
    }
}
