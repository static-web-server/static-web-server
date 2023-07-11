// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Redirection module to handle config redirect URLs with pattern matching support.
//!

use crate::settings::Redirects;

/// It returns a redirect's destination path and status code if the current request uri
/// matches against the provided redirect's array.
pub fn get_redirection<'a>(
    uri_path: &'a str,
    redirects_opts_vec: &'a Option<Vec<Redirects>>,
) -> Option<&'a Redirects> {
    if let Some(redirects_vec) = redirects_opts_vec {
        for redirect_entry in redirects_vec.iter() {
            // Match source glob pattern against the request uri path
            if redirect_entry.source.is_match(uri_path) {
                return Some(redirect_entry);
            }
        }
    }

    None
}
