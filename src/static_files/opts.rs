// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Options and response types for the static-files handler.

use headers::{HeaderMap, HeaderValue};
use hyper::{Method, Response};
use std::path::PathBuf;

use crate::body::Body;

#[cfg(feature = "mem-cache")]
use crate::mem_cache::cache::MemCacheOpts;

#[cfg(feature = "directory-listing")]
use crate::directory_listing::DirListFmt;

#[cfg(feature = "directory-listing-download")]
use crate::directory_listing::download::DirDownloadFmt;

/// Default index file used when no index files are configured.
pub(super) const DEFAULT_INDEX_FILES: &[&str; 1] = &["index.html"];

/// Defines all options needed by the static-files handler.
pub struct HandleOpts<'a> {
    /// Request method.
    pub method: &'a Method,
    /// In-memory files cache feature.
    #[cfg(feature = "mem-cache")]
    pub memory_cache: Option<&'a MemCacheOpts>,
    /// Request headers.
    pub headers: &'a HeaderMap<HeaderValue>,
    /// Request base path.
    pub base_path: &'a PathBuf,
    /// Request base path.
    pub uri_path: &'a str,
    /// Index files.
    pub index_files: &'a [&'a str],
    /// Request URI query.
    pub uri_query: Option<&'a str>,
    /// Directory listing feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing: bool,
    /// Directory listing order feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing_order: u8,
    /// Directory listing format feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing_format: &'a DirListFmt,
    /// Directory listing download feature.
    #[cfg(feature = "directory-listing-download")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing-download")))]
    pub dir_listing_download: &'a [DirDownloadFmt],
    /// Redirect trailing slash feature.
    pub redirect_trailing_slash: bool,
    /// Compression static feature.
    pub compression_static: bool,
    /// Weak ETag header feature.
    pub etag: bool,
    /// Ignore hidden files feature.
    pub include_hidden: bool,
    /// Prevent following symlinks for files and directories.
    pub follow_symlinks: bool,
}

/// Static file response type with additional data.
pub struct StaticFileResponse {
    /// Inner HTTP response.
    pub resp: Response<Body>,
    /// The file path of the inner HTTP response.
    pub file_path: PathBuf,
}

impl StaticFileResponse {
    /// Pairs a response body with the file path that produced it.
    pub(super) fn new(resp: Response<Body>, file_path: PathBuf) -> Self {
        Self { resp, file_path }
    }
}
