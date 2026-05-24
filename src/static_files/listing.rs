// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Directory-listing dispatch for the static-files handler.
//!
//! Activated when:
//!   * `--directory-listing` is enabled, and
//!   * the request maps to a directory that does not contain an index file.
//!
//! When the `directory-listing-download` feature is also enabled and the
//! request carries a `?download=<fmt>` parameter, the listing is replaced
//! with an on-the-fly archive download.

#![cfg(feature = "directory-listing")]

use hyper::{Response, StatusCode};
use std::path::Path;

use crate::body::Body;
use crate::directory_listing::{self, DirListOpts};

#[cfg(feature = "directory-listing-download")]
use crate::directory_listing::download::{DOWNLOAD_PARAM_KEY, DirDownloadOpts, archive_reply};

use super::opts::HandleOpts;

/// Returns a directory-listing (or archive download) response when applicable,
/// `Ok(None)` otherwise.
pub(super) fn try_listing(
    file_path: &Path,
    is_dir: bool,
    opts: &HandleOpts<'_>,
) -> Result<Option<Response<Body>>, StatusCode> {
    if !(is_dir && opts.dir_listing && !file_path.exists()) {
        return Ok(None);
    }

    #[cfg(feature = "directory-listing-download")]
    if let Some(resp) = try_archive_download(file_path, opts)? {
        return Ok(Some(resp));
    }

    let resp = directory_listing::auto_index(DirListOpts {
        root_path: opts.base_path.as_path(),
        method: opts.method,
        current_path: opts.uri_path,
        uri_query: opts.uri_query,
        filepath: file_path,
        dir_listing_order: opts.dir_listing_order,
        dir_listing_format: opts.dir_listing_format,
        include_hidden: opts.include_hidden,
        follow_symlinks: opts.follow_symlinks,
        #[cfg(feature = "directory-listing-download")]
        dir_listing_download: opts.dir_listing_download,
    })?;

    Ok(Some(resp))
}

/// Inspects the query string for a `download=<fmt>` parameter and, when
/// present, produces a streaming archive of the directory contents.
#[cfg(feature = "directory-listing-download")]
fn try_archive_download(
    file_path: &Path,
    opts: &HandleOpts<'_>,
) -> Result<Option<Response<Body>>, StatusCode> {
    if opts.dir_listing_download.is_empty() {
        return Ok(None);
    }

    let download_requested = form_urlencoded::parse(opts.uri_query.unwrap_or("").as_bytes())
        .any(|(k, _)| k == DOWNLOAD_PARAM_KEY);
    if !download_requested {
        return Ok(None);
    }

    // `file_path` points to the (non-existing) `<dir>/index.html`. Strip
    // the appended index segment to recover the directory path.
    let mut dir_path = file_path.to_path_buf();
    dir_path.pop();

    let Some(filename) = dir_path.file_name() else {
        tracing::error!("Unable to get filename from {}", dir_path.to_string_lossy());
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let resp = archive_reply(
        filename,
        &dir_path,
        DirDownloadOpts {
            method: opts.method,
            follow_symlinks: opts.follow_symlinks,
            include_hidden: opts.include_hidden,
        },
    );
    Ok(Some(resp))
}
