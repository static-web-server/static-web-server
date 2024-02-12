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
        Settings,
    };

    /// Testing Remote address
    pub const REMOTE_ADDR: &str = "127.0.0.1:1234";

    /// Create a `RequestHandler` from a custom TOML config file (fixture).
    pub fn fixture_req_handler(fixture_toml: &str) -> RequestHandler {
        // Replace default config file and load the fixture TOML settings
        let f = PathBuf::from("tests/fixtures").join(fixture_toml);
        std::env::set_var("SERVER_CONFIG_FILE", f);
        let opts = Settings::get(false).unwrap();

        let req_handler_opts = RequestHandlerOpts {
            root_dir: opts.general.root,
            compression: opts.general.compression,
            compression_static: opts.general.compression_static,
            #[cfg(feature = "directory-listing")]
            dir_listing: opts.general.directory_listing,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: opts.general.directory_listing_order,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: opts.general.directory_listing_format,
            // TODO: add support or `cors` when required
            cors: None,
            security_headers: opts.general.security_headers,
            cache_control_headers: opts.general.cache_control_headers,
            page404: opts.general.page404,
            page50x: opts.general.page50x,
            // TODO: add support or `page_fallback` when required
            #[cfg(feature = "fallback-page")]
            page_fallback: vec![],
            #[cfg(feature = "basic-auth")]
            basic_auth: opts.general.basic_auth,
            log_remote_address: opts.general.log_remote_address,
            redirect_trailing_slash: opts.general.redirect_trailing_slash,
            ignore_hidden_files: opts.general.ignore_hidden_files,
            index_files: vec![opts.general.index_files],
            health: opts.general.health,
            #[cfg(all(unix, feature = "experimental"))]
            experimental_metrics: opts.general.experimental_metrics,
            maintenance_mode: opts.general.maintenance_mode,
            maintenance_mode_status: opts.general.maintenance_mode_status,
            maintenance_mode_file: opts.general.maintenance_mode_file,
            advanced_opts: opts.advanced,
        };

        RequestHandler {
            opts: Arc::from(req_handler_opts),
        }
    }
}
