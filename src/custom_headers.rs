// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to append custom HTTP headers via TOML config file.
//!

use hyper::{Body, Response};

use crate::settings::Headers;

/// Append custom HTTP headers to current response.
#[inline]
pub fn append_headers(uri_path: &str, headers_opts: Option<&[Headers]>, resp: &mut Response<Body>) {
    if let Some(headers_vec) = headers_opts {
        for headers_entry in headers_vec.iter() {
            // Match header glob pattern against request uri
            if headers_entry.source.is_match(uri_path) {
                // Add/update headers if uri matches
                for (name, value) in &headers_entry.headers {
                    resp.headers_mut().insert(name, value.to_owned());
                }
            }
        }
    }
}
