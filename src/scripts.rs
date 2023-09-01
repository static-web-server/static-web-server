// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that checks for Lua script file path with pattern matching support.
//!

use crate::settings::Scripts;

/// It returns a script's destination path if the current request uri
/// matches against the provided scripts array.
pub fn script_uri_path<'a>(
    uri_path: &'a str,
    scripts_opts_vec: &'a Option<Vec<Scripts>>,
) -> Option<&'a Scripts> {
    if let Some(scripts_vec) = scripts_opts_vec {
        for scripts_entry in scripts_vec.iter() {
            // Match source glob pattern against request uri path
            if scripts_entry.source.is_match(uri_path) {
                return Some(scripts_entry);
            }
        }
    }

    None
}
