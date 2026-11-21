// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Options and response types for the static-files handler.

use headers::{HeaderMap, HeaderValue};
use hyper::{Method, Response, Uri};
use std::path::PathBuf;

use crate::body::Body;
use crate::rewrites;
use crate::settings::Rewrites;

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
    /// Request URI before a rewrite changed it to `uri_path` and
    /// `uri_query`, `None` when no rewrite applied.
    pub rewritten_from: Option<RewrittenFrom<'a>>,
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

impl HandleOpts<'_> {
    /// Returns the path and query a trailing-slash redirect is decided on
    /// and points to.
    ///
    /// They are the client's when a rewrite changed them, the client's path
    /// doesn't end with `/`, and that path with `/` appended rewrites to the
    /// same directory, so the redirect keeps the rewrite destination hidden.
    /// Otherwise they are `uri_path` and `uri_query`, as the client already
    /// sent a `/` or a redirect to its path wouldn't reach the directory.
    /// Matches the rewrite rules again, so call it only when a redirect is
    /// possible.
    pub(super) fn redirect_path_and_query(&self) -> (&str, Option<&str>) {
        match &self.rewritten_from {
            Some(from)
                if !from.uri.path().ends_with('/')
                    && rewrites::has_same_destination_with_slash(from.rules, from.uri.path()) =>
            {
                (from.uri.path(), from.uri.query())
            }
            _ => (self.uri_path, self.uri_query),
        }
    }
}

/// Request URI as the client sent it, with the rewrite rules that changed it.
pub struct RewrittenFrom<'a> {
    /// Request URI as the client sent it.
    pub uri: &'a Uri,
    /// Rewrite rules from the config file.
    pub rules: &'a [Rewrites],
}

/// Static file response type with additional data.
pub struct StaticFileResponse {
    /// Inner HTTP response.
    pub resp: Response<Body>,
    /// The file path of the inner HTTP response, `None` for an `OPTIONS` reply.
    pub file_path: Option<PathBuf>,
    /// Whether the inner HTTP response is a directory listing.
    pub is_dir_listing: bool,
}

impl StaticFileResponse {
    /// Pairs a response body with the file path that produced it.
    pub(super) fn new(resp: Response<Body>, file_path: PathBuf) -> Self {
        Self {
            resp,
            file_path: Some(file_path),
            is_dir_listing: false,
        }
    }
}
