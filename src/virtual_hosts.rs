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
    if let Some(vhosts) = vhosts_opts {
        if let Ok(host_str) = req.headers().get(HOST)?.to_str() {
            for vhost in vhosts {
                if vhost.host == host_str {
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
        }
    }
    None
}
