// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

use crate::directory_listing::file::FileEntry;

/// Defines sorting attributes for file entries.
pub(crate) struct SortingAttr<'a> {
    pub(crate) name: &'a str,
    pub(crate) last_modified: &'a str,
    pub(crate) size: &'a str,
}

/// Sort a list of file entries by a specific order code.
pub(crate) fn sort_file_entries(files: &mut [FileEntry], order_code: u8) -> SortingAttr<'_> {
    // Default sorting type values
    let mut name = "0";
    let mut last_modified = "2";
    let mut size = "4";

    match order_code {
        0 | 1 => {
            // Name (asc, desc)
            files.sort_by_cached_key(|f| f.name.to_string_lossy().to_lowercase());
            if order_code == 1 {
                files.reverse();
            } else {
                name = "1";
            }
        }
        2 | 3 => {
            // Modified (asc, desc)
            files.sort_by_key(|f| f.mtime);
            if order_code == 3 {
                files.reverse();
            } else {
                last_modified = "3";
            }
        }
        4 | 5 => {
            // File size (asc, desc)
            files.sort_by_key(|f| f.size);
            if order_code == 5 {
                files.reverse();
            } else {
                size = "5";
            }
        }
        _ => {
            // Unsorted
        }
    }

    SortingAttr {
        name,
        last_modified,
        size,
    }
}
