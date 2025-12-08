// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

mod html;
mod json;

pub(crate) use html::html_auto_index;
pub(crate) use json::json_auto_index;

use hyper::{Body, Response, StatusCode};
use std::io;

use crate::Result;
use crate::directory_listing::dir::{DirEntryOpts, DirListOpts, read_dir_entries};
use crate::http_ext::MethodExt;

/// Provides directory listing support for the current request.
/// Note that this function highly depends on `static_files::composed_file_metadata()` function
/// which must be called first. See `static_files::handle()` for more details.
pub fn auto_index(opts: DirListOpts<'_>) -> Result<Response<Body>, StatusCode> {
    // Note: it's safe to call `parent()` here since `filepath`
    // value always refer to a path with file ending and under
    // a root directory boundary.
    // See `composed_file_metadata()` function which sanitizes the requested
    // path before to be delegated here.
    let filepath = opts.filepath;
    let parent = filepath.parent().unwrap_or(filepath);

    match std::fs::read_dir(parent) {
        Ok(dir_reader) => {
            let dir_opts = DirEntryOpts {
                root_path: opts.root_path,
                dir_reader,
                base_path: opts.current_path,
                uri_query: opts.uri_query,
                is_head: opts.method.is_head(),
                order_code: opts.dir_listing_order,
                content_format: opts.dir_listing_format,
                ignore_hidden_files: opts.ignore_hidden_files,
                disable_symlinks: opts.disable_symlinks,
                #[cfg(feature = "directory-listing-download")]
                download: opts.dir_listing_download,
            };
            match read_dir_entries(dir_opts) {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    tracing::error!("error after try to read directory entries: {:?}", err);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!(
                        "entry file not found (path: {}): {:?}",
                        filepath.display(),
                        err
                    );
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::error!(
                        "entry file permission denied (path: {}): {:?}",
                        filepath.display(),
                        err
                    );
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!(
                        "unable to read parent directory (parent={}): {:?}",
                        parent.display(),
                        err
                    );
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Err(status)
        }
    }
}
