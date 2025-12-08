// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

use crate::Result;
use crate::directory_listing::file::FileEntry;
use crate::directory_listing::sort::sort_file_entries;

/// Create an auto index in JSON format.
pub(crate) fn json_auto_index(entries: &mut [FileEntry], order_code: u8) -> Result<String> {
    sort_file_entries(entries, order_code);

    Ok(serde_json::to_string(entries)?)
}
