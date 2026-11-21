// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Options shared by the request handler and the feature modules it runs.
//!

use hyper::StatusCode;
use std::{
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use crate::{access_log_format::AccessLogFormat, cors, settings::Advanced};

#[cfg(feature = "directory-listing")]
use crate::settings::file::DirListFmt;

#[cfg(feature = "directory-listing-download")]
use crate::settings::file::DirDownloadFmt;

/// It defines the in-memory files cache options.
#[cfg(feature = "mem-cache")]
#[cfg_attr(docsrs, doc(cfg(feature = "mem-cache")))]
pub struct MemCacheOpts {
    /// The maximum size per file in bytes.
    pub max_file_size: u64,
}

#[cfg(feature = "mem-cache")]
impl MemCacheOpts {
    /// Creates a new instance of `MemCacheOpts`.
    /// `max_file_size` is in KiB and gets converted to bytes internally.
    #[inline]
    pub fn new(max_file_size: u64) -> Self {
        Self {
            max_file_size: max_file_size * 1024,
        }
    }
}

/// It defines options for a request handler.
pub struct RequestHandlerOpts {
    // General options
    /// Root directory of static files.
    pub root_dir: PathBuf,
    #[cfg(feature = "mem-cache")]
    /// In-memory cache feature.
    pub memory_cache: Option<MemCacheOpts>,
    /// Compression feature.
    pub compression: bool,
    #[cfg(any(
        feature = "compression",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd",
        feature = "compression-deflate"
    ))]
    /// Compression level.
    pub compression_level: crate::settings::CompressionLevel,
    /// Compression static feature.
    pub compression_static: bool,
    /// Directory listing feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing: bool,
    /// Directory listing order feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing_order: u8,
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    /// Directory listing format feature.
    pub dir_listing_format: DirListFmt,
    /// Directory listing download feature.
    #[cfg(feature = "directory-listing-download")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing-download")))]
    pub dir_listing_download: Vec<DirDownloadFmt>,
    /// CORS feature.
    pub cors: Option<cors::Configured>,
    /// Security headers feature.
    pub security_headers: bool,
    /// Send `Strict-Transport-Security` with the security headers. Only set when the server uses TLS.
    pub hsts: bool,
    /// Cache control headers feature.
    pub cache_control_headers: bool,
    /// Weak ETag header feature.
    pub etag: bool,
    /// Page for 404 errors.
    pub page404: PathBuf,
    /// Page for 50x errors.
    pub page50x: PathBuf,
    /// Page fallback feature.
    #[cfg(feature = "fallback-page")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fallback-page")))]
    pub page_fallback: Vec<u8>,
    /// Basic auth feature.
    #[cfg(feature = "basic-auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "basic-auth")))]
    pub basic_auth: String,
    /// Index files feature.
    pub index_files: Vec<String>,
    /// Compiled access log format.
    pub access_log_format: Option<AccessLogFormat>,
    /// Log remote address feature.
    pub log_remote_address: bool,
    /// Log the X-Real-IP header.
    pub log_x_real_ip: bool,
    /// Log the client address from the X-Forwarded-For header,
    /// the rightmost address not in `trusted_proxies`. Entries must be bare IP addresses.
    pub log_forwarded_for: bool,
    /// Proxy IPs (exact, no ranges) to log the X-Real-IP and X-Forwarded-For headers from, none if empty.
    pub trusted_proxies: Vec<IpAddr>,
    /// Redirect trailing slash feature.
    pub redirect_trailing_slash: bool,
    /// Ignore hidden files feature.
    pub include_hidden: bool,
    /// Prevent following symlinks for files and directories.
    pub follow_symlinks: bool,
    /// Resolve the web root directory at request time rather than at startup.
    pub use_relative_root: bool,
    /// Accept markdown content negotiation feature.
    pub accept_markdown: bool,
    /// Default `charset=utf-8` parameter applied to certain `text` responses without one.
    pub text_charset: bool,
    /// Health endpoints feature (`/livez` and `/readyz`).
    pub health: bool,
    /// Set once a shutdown signal is received; `/readyz` then returns `503`.
    pub shutting_down: Arc<AtomicBool>,
    /// Metrics endpoint feature.
    #[cfg(feature = "metrics")]
    pub metrics_enabled: bool,
    /// Maintenance mode feature.
    pub maintenance_mode: bool,
    /// Custom HTTP status for when entering into maintenance mode.
    pub maintenance_mode_status: StatusCode,
    /// Custom maintenance mode HTML file.
    pub maintenance_mode_file: PathBuf,

    /// Advanced options from the config file.
    pub advanced_opts: Option<Advanced>,
}

impl Default for RequestHandlerOpts {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("./public"),
            compression: true,
            compression_static: false,
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression_level: crate::settings::CompressionLevel::Default,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6, // unordered
            #[cfg(feature = "directory-listing")]
            dir_listing_format: DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: Vec::new(),
            cors: None,
            #[cfg(feature = "mem-cache")]
            memory_cache: None,
            security_headers: false,
            hsts: false,
            cache_control_headers: true,
            etag: true,
            page404: PathBuf::from("./404.html"),
            page50x: PathBuf::from("./50x.html"),
            #[cfg(feature = "fallback-page")]
            page_fallback: Vec::new(),
            #[cfg(feature = "basic-auth")]
            basic_auth: String::new(),
            index_files: vec!["index.html".into()],
            access_log_format: None,
            log_remote_address: false,
            log_x_real_ip: false,
            log_forwarded_for: false,
            trusted_proxies: Vec::new(),
            redirect_trailing_slash: true,
            include_hidden: true,
            follow_symlinks: true,
            use_relative_root: false,
            accept_markdown: false,
            text_charset: true,
            health: false,
            shutting_down: Arc::default(),
            #[cfg(feature = "metrics")]
            metrics_enabled: false,
            maintenance_mode: false,
            maintenance_mode_status: StatusCode::SERVICE_UNAVAILABLE,
            maintenance_mode_file: PathBuf::new(),
            advanced_opts: None,
        }
    }
}
