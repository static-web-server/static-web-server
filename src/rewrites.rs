// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that allows to rewrite request URLs with pattern matching support.
//!

use crate::settings::Rewrites;

/// It returns a rewrite's destination path if the current request uri
/// matches against the provided rewrites array.
#[inline]
pub fn rewrite_uri_path<'a>(
    uri_path: &'a str,
    rewrites_opts: Option<&'a [Rewrites]>,
) -> Option<&'a Rewrites> {
    if let Some(rewrites_vec) = rewrites_opts {
        for rewrites_entry in rewrites_vec.iter() {
            // Match source glob pattern against request uri path
            if rewrites_entry.source.is_match(uri_path) {
                return Some(rewrites_entry);
            }
        }
    }

    None
}
