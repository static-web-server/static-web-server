// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Compression static module to serve compressed files directly from the file system.
//!

use headers::{HeaderMap, HeaderValue};
use hyper::{Body, Request, Response};
use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::{Path, PathBuf};

use crate::compression;
use crate::fs::meta::try_metadata;
use crate::handler::RequestHandlerOpts;
use crate::headers_ext::ContentCoding;
use crate::Error;

/// It defines the pre-compressed file variant metadata of a particular file path.
pub struct CompressedFileVariant {
    /// Current file path.
    pub file_path: PathBuf,
    /// The metadata of the current file.
    pub metadata: Metadata,
    /// The content encoding based on the file extension.
    pub encoding: ContentCoding,
}

/// Initializes static compression.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.compression_static = enabled;
    server_info!("compression static: enabled={enabled}");
}

/// Post-processing to add Vary header if necessary.
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    _req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if !opts.compression_static {
        return Ok(resp);
    }

    // Compression content encoding varies so use a `Vary` header
    resp.headers_mut().insert(
        hyper::header::VARY,
        HeaderValue::from_name(hyper::header::ACCEPT_ENCODING),
    );

    Ok(resp)
}

/// Search for the pre-compressed variant of the given file path.
pub fn precompressed_variant(
    file_path: &Path,
    headers: &HeaderMap<HeaderValue>,
) -> Option<CompressedFileVariant> {
    tracing::trace!(
        "preparing pre-compressed file variant path of {}",
        file_path.display()
    );

    for encoding in compression::get_encodings(headers) {
        // Determine preferred-encoding extension if available
        let comp_ext = match encoding {
            // https://zlib.net/zlib_faq.html#faq39
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-deflate"
            ))]
            ContentCoding::GZIP | ContentCoding::DEFLATE => "gz",
            // https://peazip.github.io/brotli-compressed-file-format.html
            #[cfg(any(feature = "compression", feature = "compression-brotli"))]
            ContentCoding::BROTLI => "br",
            // https://datatracker.ietf.org/doc/html/rfc8878
            #[cfg(any(feature = "compression", feature = "compression-zstd"))]
            ContentCoding::ZSTD => "zst",
            _ => {
                tracing::trace!(
                    "preferred encoding based on the file extension was not determined, skipping"
                );
                continue;
            }
        };

        let comp_name = match file_path.file_name().and_then(OsStr::to_str) {
            Some(v) => v,
            None => {
                tracing::trace!("file name was not determined for the current path, skipping");
                continue;
            }
        };

        let file_path = file_path.with_file_name([comp_name, ".", comp_ext].concat());
        tracing::trace!(
            "trying to get the pre-compressed file variant metadata for {}",
            file_path.display()
        );

        let (metadata, is_dir) = match try_metadata(&file_path) {
            Ok(v) => v,
            Err(e) => {
                tracing::trace!("pre-compressed file variant error: {:?}", e);
                continue;
            }
        };

        if is_dir {
            tracing::trace!("pre-compressed file variant found but it's a directory, skipping");
            continue;
        }

        tracing::trace!("pre-compressed file variant found, serving it directly");

        return Some(CompressedFileVariant {
            file_path,
            metadata,
            encoding,
        });
    }

    None
}
