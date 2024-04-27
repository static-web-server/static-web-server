// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Compression static module to serve compressed files directly from the file system.
//!

use headers::{HeaderMap, HeaderValue};
use hyper::{Body, Request, Response};
use std::{
    ffi::OsStr,
    fs::Metadata,
    path::{Path, PathBuf},
};

use crate::{
    compression, handler::RequestHandlerOpts, headers_ext::ContentCoding,
    static_files::file_metadata,
};

/// It defines the pre-compressed file variant metadata of a particular file path.
pub struct CompressedFileVariant<'a> {
    /// Current file path.
    pub file_path: PathBuf,
    /// The metadata of the current file.
    pub metadata: Metadata,
    /// The file extension.
    pub extension: &'a str,
}

/// Initializes static compression.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.compression_static = enabled;
    server_info!("compression static: enabled={enabled}");
}

/// Post-processing to add Vary header if necessary.
pub(crate) fn post_process(
    opts: &RequestHandlerOpts,
    _req: &Request<Body>,
    resp: &mut Response<Body>,
) {
    if !opts.compression_static {
        return;
    }

    // Compression content encoding varies so use a `Vary` header
    resp.headers_mut().append(
        hyper::header::VARY,
        HeaderValue::from_name(hyper::header::ACCEPT_ENCODING),
    );
}

/// Search for the pre-compressed variant of the given file path.
pub async fn precompressed_variant<'a>(
    file_path: &Path,
    headers: &'a HeaderMap<HeaderValue>,
) -> Option<CompressedFileVariant<'a>> {
    tracing::trace!(
        "preparing pre-compressed file variant path of {}",
        file_path.display()
    );

    // Determine preferred-encoding extension if available
    let comp_ext = match compression::get_preferred_encoding(headers) {
        // https://zlib.net/zlib_faq.html#faq39
        #[cfg(any(feature = "compression", feature = "compression-gzip"))]
        Some(ContentCoding::GZIP | ContentCoding::DEFLATE) => "gz",
        // https://peazip.github.io/brotli-compressed-file-format.html
        #[cfg(any(feature = "compression", feature = "compression-brotli"))]
        Some(ContentCoding::BROTLI) => "br",
        // https://datatracker.ietf.org/doc/html/rfc8878
        #[cfg(any(feature = "compression", feature = "compression-zstd"))]
        Some(ContentCoding::ZSTD) => "zst",
        _ => {
            tracing::trace!(
                "preferred encoding based on the file extension was not determined, skipping"
            );
            return None;
        }
    };

    let comp_name = match file_path.file_name().and_then(OsStr::to_str) {
        Some(v) => v,
        None => {
            tracing::trace!("file name was not determined for the current path, skipping");
            return None;
        }
    };

    let file_path = file_path.with_file_name([comp_name, ".", comp_ext].concat());
    tracing::trace!(
        "trying to get the pre-compressed file variant metadata for {}",
        file_path.display()
    );

    let (metadata, is_dir) = match file_metadata(&file_path) {
        Ok(v) => v,
        Err(e) => {
            tracing::trace!("pre-compressed file variant error: {:?}", e);
            return None;
        }
    };

    if is_dir {
        tracing::trace!("pre-compressed file variant found but it's a directory, skipping");
        return None;
    }

    tracing::trace!("pre-compressed file variant found, serving it directly");

    Some(CompressedFileVariant {
        file_path,
        metadata,
        extension: if comp_ext == "gz" { "gzip" } else { comp_ext },
    })
}
