// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that provides directory listing and auto-index support.
//!

mod autoindex;
mod dir;
mod file;
mod sort;
mod style;

pub(crate) use autoindex::*;
pub use dir::*;

use crate::handler::RequestHandlerOpts;

/// Initializes directory listings.
pub fn init(enabled: bool, order: u8, format: DirListFmt, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.dir_listing = enabled;
    tracing::info!("directory listing: enabled={enabled}");

    handler_opts.dir_listing_order = order;
    tracing::info!("directory listing order code: {order}");

    handler_opts.dir_listing_format = format;
    tracing::info!(
        "directory listing format: {:?}",
        handler_opts.dir_listing_format
    );
}
