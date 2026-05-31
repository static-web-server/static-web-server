// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! File metadata and index resolution for the static-files handler.
//!
//! This module owns the logic that, given a sanitized request path, picks
//! the actual file to serve: directory → index-file resolution,
//! `.html`-suffix fallback, and pre-compressed variant lookup.

use headers::{HeaderMap, HeaderValue};
use hyper::StatusCode;
use std::fs::Metadata;
use std::path::PathBuf;

use crate::Result;
use crate::compression_static;
use crate::fs::meta::{FileMetadata, try_file_open, try_metadata, try_metadata_with_html_suffix};

use super::opts::DEFAULT_INDEX_FILES;

/// Resolves the file to serve for a sanitized request path.
///
/// Returns the final file path along with its metadata and, when applicable,
/// the pre-compressed variant that should be served instead.
pub(super) fn file_metadata<'a>(
    mut file_path: &'a mut PathBuf,
    headers: &'a HeaderMap<HeaderValue>,
    compression_static: bool,
    mut index_files: &'a [&'a str],
) -> Result<FileMetadata<'a>, StatusCode> {
    tracing::trace!("getting metadata for file {}", file_path.display());

    match try_metadata(file_path) {
        Ok((mut metadata, is_dir)) => {
            // The optional pre-opened file for `file_path`. When `Some`, the
            // response pipeline reuses this handle instead of issuing an
            // extra `open(2)` syscall. We only populate it when the index
            // file is resolved via `try_file_open` below.
            let mut opened_file = None;
            // Whether the resolved `file_path` points to an existing file.
            // For non-directory requests this is always true.
            // For directory requests it becomes true only when an index file (or its
            // `.html` suffix sibling) was successfully resolved. Used to
            // gate the pre-compressed variant probe so we never issue
            // `stat(2)` for `.br`/`.gz`/`.zst` siblings of a non-existent
            // index (see issue #617).
            let mut resolved_exists = !is_dir;
            if is_dir {
                if index_files.is_empty() {
                    index_files = DEFAULT_INDEX_FILES;
                }
                for index in index_files {
                    // Append a HTML index page by default if it's a directory path (`autoindex`).
                    tracing::debug!("dir: appending {} to the directory path", index);
                    file_path.push(index);

                    // Try to open the appended index file directly.
                    // `try_file_open` performs a single `open(2)` + `fstat(2)`
                    // instead of `stat(2)` followed by `open(2)` later in
                    // `file_reply`, saving one path-resolving syscall on the
                    // hot path.
                    if let Ok((file, meta)) = try_file_open(file_path) {
                        metadata = meta;
                        opened_file = Some(file);
                        resolved_exists = true;
                        break;
                    }

                    // Remove only the appended index file before trying the
                    // `.html` suffix fallback against the directory path.
                    file_path.pop();
                    let new_meta: Option<Metadata>;
                    (file_path, new_meta) = try_metadata_with_html_suffix(file_path);
                    if let Some(new_meta) = new_meta {
                        metadata = new_meta;
                        resolved_exists = true;
                        break;
                    }
                }

                // If no index was found, append the last index of the list
                // to preserve the original directory-listing behavior.
                if !resolved_exists && !index_files.is_empty() {
                    file_path.push(index_files.last().unwrap());
                }
            }

            // Only probe for pre-compressed siblings when the resolved file
            // actually exists. Probing for `.br`/`.gz`/`.zst` of a path that
            // was never confirmed on disk wastes one `stat(2)` per
            // configured encoding on the request hot path
            // (see issue #617).
            let precompressed_variant = (compression_static && resolved_exists)
                .then(|| compression_static::precompressed_variant(file_path, headers))
                .flatten()
                .map(|p| (p.file_path, p.encoding));

            // If we are going to serve a precompressed variant, the
            // pre-opened file points to the *original* file which won't be
            // streamed; drop it so `file_reply` opens the precomp file.
            if precompressed_variant.is_some() {
                opened_file = None;
            }

            Ok(FileMetadata {
                file_path,
                metadata,
                is_dir,
                precompressed_variant,
                file: opened_file,
            })
        }
        Err(err) => {
            // If the file path doesn't exist, then try the `.html`-suffixed path
            // first. For example: `/posts/article` falls back to
            // `/posts/article.html`.
            //
            // We intentionally do *not* probe for pre-compressed siblings
            // of the original (non-existent) path. Doing so would waste
            // one `stat(2)` per configured encoding for every truly
            // missing path (see issue #617).
            let new_meta: Option<Metadata>;
            (file_path, new_meta) = try_metadata_with_html_suffix(file_path);

            let Some(new_meta) = new_meta else {
                // Neither the original path nor its `.html` sibling exists.
                // Return the original error without probing for compressed
                // variants of non-existent files.
                return Err(err);
            };

            // The `.html` sibling exists. Only now is it worth probing for
            // its pre-compressed sibling (`/article.html.br`, etc.).
            let precompressed_variant = compression_static
                .then(|| compression_static::precompressed_variant(file_path, headers))
                .flatten()
                .map(|p| (p.file_path, p.encoding));

            Ok(FileMetadata {
                file_path,
                metadata: new_meta,
                is_dir: false,
                precompressed_variant,
                file: None,
            })
        }
    }
}
