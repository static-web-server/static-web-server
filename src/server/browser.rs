// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Default browser launcher for the TCP server implementations.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub(super) fn listener_url(
    scheme: &str,
    listener: &tokio::net::TcpListener,
    path: Option<&str>,
) -> Option<String> {
    match listener.local_addr() {
        Ok(addr) => Some(browser_url(scheme, addr, path)),
        Err(err) => {
            tracing::warn!(%err, "failed to determine listener address for browser URL");
            None
        }
    }
}

pub(super) fn open(url: String) {
    let spawn_result = std::thread::Builder::new()
        .name("sws-browser".to_owned())
        .spawn(move || match open::that(&url) {
            Ok(()) => {
                tracing::info!(%url, "opened server URL in web browser");
            }
            Err(err) => {
                tracing::warn!(%url, %err, "failed to open server URL in web browser");
            }
        });

    if let Err(err) = spawn_result {
        tracing::warn!(%err, "failed to spawn browser launcher thread");
    }
}

#[must_use]
fn browser_url(scheme: &str, mut addr: SocketAddr, path: Option<&str>) -> String {
    if addr.ip().is_unspecified() {
        let loopback = match addr {
            SocketAddr::V4(_) => IpAddr::V4(Ipv4Addr::LOCALHOST),
            SocketAddr::V6(_) => IpAddr::V6(Ipv6Addr::LOCALHOST),
        };
        addr.set_ip(loopback);
    }

    match path {
        Some(path) if path.starts_with('/') => format!("{scheme}://{addr}{path}"),
        Some(path) => format!("{scheme}://{addr}/{path}"),
        None => format!("{scheme}://{addr}/"),
    }
}

#[cfg(test)]
mod tests {
    use super::browser_url;

    #[test]
    fn ipv4_unspecified_address_uses_ipv4_loopback() {
        let addr = "0.0.0.0:8080".parse().unwrap();
        assert_eq!(browser_url("http", addr, None), "http://127.0.0.1:8080/");
    }

    #[test]
    fn ipv6_unspecified_address_uses_ipv6_loopback() {
        let addr = "[::]:8080".parse().unwrap();
        assert_eq!(browser_url("http", addr, None), "http://[::1]:8080/");
    }

    #[test]
    fn https_url_preserves_bound_port() {
        let addr = "127.0.0.1:8443".parse().unwrap();
        assert_eq!(browser_url("https", addr, None), "https://127.0.0.1:8443/");
    }

    #[test]
    fn configured_path_is_appended_to_url() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        assert_eq!(
            browser_url("http", addr, Some("/docs")),
            "http://127.0.0.1:8080/docs"
        );
    }

    #[test]
    fn configured_path_without_leading_slash_is_normalized() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        assert_eq!(
            browser_url("http", addr, Some("docs")),
            "http://127.0.0.1:8080/docs"
        );
    }
}
