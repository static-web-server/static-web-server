// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Development utilities for testing of SWS.
//!

/// SWS fixtures module.
#[doc(hidden)]
pub mod fixtures {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        handler::{RequestHandler, RequestHandlerOpts},
        settings::cli::General,
        settings::Advanced,
        Settings,
    };

    /// Testing Remote address
    pub const REMOTE_ADDR: &str = "127.0.0.1:1234";

    /// Load the fixture TOML settings and return it.
    pub fn fixture_settings(fixture_toml: &str) -> Settings {
        // Replace default config file and load the fixture TOML settings
        let f = PathBuf::from("tests/fixtures").join(fixture_toml);
        std::env::set_var("SERVER_CONFIG_FILE", f);
        Settings::get_unparsed(false).unwrap()
    }

    /// Create a `RequestHandler` from a custom TOML config file (fixture).
    pub fn fixture_req_handler(general: General, advanced: Option<Advanced>) -> RequestHandler {
        #[cfg(not(any(
            feature = "compression",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
            feature = "compression-deflate"
        )))]
        let compression = false;
        #[cfg(not(any(
            feature = "compression",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
            feature = "compression-deflate"
        )))]
        let compression_static = false;
        #[cfg(any(
            feature = "compression",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
            feature = "compression-deflate"
        ))]
        let compression = general.compression;
        #[cfg(any(
            feature = "compression",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
            feature = "compression-deflate"
        ))]
        let compression_static = general.compression_static;

        let req_handler_opts = RequestHandlerOpts {
            root_dir: general.root,
            compression,
            compression_static,
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression_level: general.compression_level,
            #[cfg(feature = "directory-listing")]
            dir_listing: general.directory_listing,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: general.directory_listing_order,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: general.directory_listing_format,
            // TODO: add support or `cors` when required
            cors: None,
            security_headers: general.security_headers,
            cache_control_headers: general.cache_control_headers,
            page404: general.page404,
            page50x: general.page50x,
            // TODO: add support or `page_fallback` when required
            #[cfg(feature = "fallback-page")]
            page_fallback: vec![],
            #[cfg(feature = "basic-auth")]
            basic_auth: general.basic_auth,
            log_remote_address: general.log_remote_address,
            log_forwarded_for: general.log_forwarded_for,
            trusted_proxies: general.trusted_proxies,
            redirect_trailing_slash: general.redirect_trailing_slash,
            ignore_hidden_files: general.ignore_hidden_files,
            disable_symlinks: general.disable_symlinks,
            index_files: vec![general.index_files],
            health: general.health,
            #[cfg(all(unix, feature = "experimental"))]
            experimental_metrics: general.experimental_metrics,
            maintenance_mode: general.maintenance_mode,
            maintenance_mode_status: general.maintenance_mode_status,
            maintenance_mode_file: general.maintenance_mode_file,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            advanced_opts: advanced,
        };

        RequestHandler {
            opts: Arc::from(req_handler_opts),
        }
    }
}
