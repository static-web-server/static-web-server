// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that allows to determine a virtual hostname.
//!

use hyper::header::HOST;
use hyper::Request;
use std::path::PathBuf;

use crate::settings::VirtualHosts;

/// It returns different root directory if the "Host" header matches a virtual hostname.
pub(crate) fn get_real_root<'a, T>(
    req: &mut Request<T>,
    vhosts_opts: Option<&'a [VirtualHosts]>,
) -> Option<&'a PathBuf> {
    let vhosts = vhosts_opts?;

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
