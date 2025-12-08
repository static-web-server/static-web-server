// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module that provides file metadata facilities.
//!

use http::StatusCode;
use std::fs::Metadata;
use std::path::{Path, PathBuf};

use crate::Result;
use crate::headers_ext::ContentCoding;

/// It defines a composed file metadata structure containing the current file
/// and its optional pre-compressed variant.
pub(crate) struct FileMetadata<'a> {
    /// The current file path reference.
    pub file_path: &'a PathBuf,
    /// The metadata of current `file_path` by default.
    /// Note that if `precompressed_variant` has some value
    /// then the `metadata` value will correspond to the `precompressed_variant`.
    pub metadata: Metadata,
    // If either `file_path` or `precompressed_variant` is a directory.
    pub is_dir: bool,
    // The precompressed file variant for the current `file_path`.
    pub precompressed_variant: Option<(PathBuf, ContentCoding)>,
}

/// Try to find the file system metadata for the given file path or return a `Not Found` error.
pub(crate) fn try_metadata(file_path: &Path) -> Result<(Metadata, bool), StatusCode> {
    match std::fs::metadata(file_path) {
        Ok(meta) => {
            let is_dir = meta.is_dir();
            tracing::trace!("file found: {:?}; is_dir: {is_dir}", file_path);
            Ok((meta, is_dir))
        }
        Err(err) => {
            tracing::debug!("file not found: {:?} {:?}", file_path, err);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Try to append a `.html` suffix to a given file path when the file doesn't exist.
/// * When the suffixed html path exists then it mutates the path to the suffixed one and returns its `Metadata`.
/// * Otherwise, it falls back the path to its original value.
pub(crate) fn try_metadata_with_html_suffix(
    file_path: &mut PathBuf,
) -> (&mut PathBuf, Option<Metadata>) {
    tracing::debug!("file: appending .html suffix to the path");

    if let Some(filename) = file_path.file_name() {
        let owned_filename = filename.to_os_string();
        let mut owned_filename_with_html = owned_filename.clone();

        owned_filename_with_html.push(".html");
        file_path.set_file_name(owned_filename_with_html);

        if let Ok(meta_res) = try_metadata(file_path) {
            let (meta, _) = meta_res;
            return (file_path, Some(meta));
        }

        tracing::debug!(
            "file: the .html suffixed path doesn't exist, falling back to the original"
        );

        file_path.set_file_name(owned_filename);
    }

    (file_path, None)
}

/// Try to find a markdown variant for a given file path.
/// * First tries to append a `.md` suffix to the path (e.g., `/article` → `/article.md`)
/// * Then tries to append `.html.md`
/// * If that fails, tries to find `index.html.md` in the path (e.g., `/article` → `/article/index.html.md`)
/// * Returns `Some(PathBuf)` if a markdown file is found, `None` otherwise
pub(crate) fn try_markdown_variant(file_path: &Path) -> Option<PathBuf> {
    // Helper to check if a path exists and is a file
    let try_path = |path: PathBuf| -> Option<PathBuf> {
        match try_metadata(&path) {
            Ok((_, false)) => Some(path),
            _ => None,
        }
    };

    // Try suffixes first if the path has a filename
    if let Some(filename) = file_path.file_name() {
        for suffix in [".md", ".html.md"] {
            let mut path = file_path.to_path_buf();
            let mut owned_filename = filename.to_os_string();
            owned_filename.push(suffix);
            path.set_file_name(owned_filename);

            if let Some(found) = try_path(path) {
                return Some(found);
            }
        }
    }

    // Try index.html.md
    let mut path = file_path.to_path_buf();
    path.push("index.html.md");
    try_path(path)
}
