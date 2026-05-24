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
            if is_dir {
                if index_files.is_empty() {
                    index_files = DEFAULT_INDEX_FILES;
                }
                let mut index_found = false;
                for index in index_files {
                    // Append a HTML index page by default if it's a directory path (`autoindex`).
                    tracing::debug!("dir: appending {} to the directory path", index);
                    file_path.push(index);

                    if compression_static
                        && let Some(p) =
                            compression_static::precompressed_variant(file_path, headers)
                    {
                        return Ok(FileMetadata {
                            file_path,
                            metadata: p.metadata,
                            is_dir: false,
                            precompressed_variant: Some((p.file_path, p.encoding)),
                            file: None,
                        });
                    }

                    // Fallback to finding the appended index file and overwrite the
                    // current metadata. Still considered a directory request.
                    // We open the file directly here: `try_file_open` performs
                    // a single `open(2)` + `fstat(2)` instead of a `stat(2)`
                    // followed by an `open(2)` later in `file_reply`,
                    // saving one path-resolving syscall on the hot path.
                    if let Ok((file, meta)) = try_file_open(file_path) {
                        metadata = meta;
                        opened_file = Some(file);
                        index_found = true;
                        break;
                    }

                    // Remove only the appended index file before trying the
                    // `.html` suffix fallback against the directory path.
                    file_path.pop();
                    let new_meta: Option<Metadata>;
                    (file_path, new_meta) = try_metadata_with_html_suffix(file_path);
                    if let Some(new_meta) = new_meta {
                        metadata = new_meta;
                        index_found = true;
                        break;
                    }
                }

                // If no index was found, append the last index of the list
                // to preserve the original directory-listing behavior.
                if !index_found && !index_files.is_empty() {
                    file_path.push(index_files.last().unwrap());
                }
            }

            let precompressed_variant = compression_static
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
            // Pre-compressed variant check for a file that was not found.
            if compression_static
                && let Some(p) = compression_static::precompressed_variant(file_path, headers)
            {
                return Ok(FileMetadata {
                    file_path,
                    metadata: p.metadata,
                    is_dir: false,
                    precompressed_variant: Some((p.file_path, p.encoding)),
                    file: None,
                });
            }

            // Otherwise, if the file path doesn't exist try the `.html`-suffixed path.
            // For example: `/posts/article` falls back to `/posts/article.html`.
            let new_meta: Option<Metadata>;
            (file_path, new_meta) = try_metadata_with_html_suffix(file_path);

            #[cfg(any(
                feature = "compression",
                feature = "compression-deflate",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd"
            ))]
            match new_meta {
                Some(new_meta) => {
                    return Ok(FileMetadata {
                        file_path,
                        metadata: new_meta,
                        is_dir: false,
                        precompressed_variant: None,
                        file: None,
                    });
                }
                _ => {
                    // Last pre-compressed variant check for the suffixed file.
                    if compression_static
                        && let Some(p) =
                            compression_static::precompressed_variant(file_path, headers)
                    {
                        return Ok(FileMetadata {
                            file_path,
                            metadata: p.metadata,
                            is_dir: false,
                            precompressed_variant: Some((p.file_path, p.encoding)),
                            file: None,
                        });
                    }
                }
            }
            #[cfg(not(feature = "compression"))]
            if let Some(new_meta) = new_meta {
                return Ok(FileMetadata {
                    file_path,
                    metadata: new_meta,
                    is_dir: false,
                    precompressed_variant: None,
                    file: None,
                });
            }

            Err(err)
        }
    }
}
