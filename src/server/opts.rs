// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Handler options initialization from server settings.

use crate::handler::RequestHandlerOpts;
use crate::settings::Advanced;
use crate::settings::cli::General;
use crate::{
    Context, Result, compression_static, control_headers, cors, etag, health, helpers, log_addr,
    maintenance_mode, security_headers,
};

#[cfg(feature = "directory-listing")]
use crate::directory_listing;

#[cfg(feature = "directory-listing-download")]
pub(crate) use crate::directory_listing::download;

#[cfg(feature = "fallback-page")]
use crate::fallback_page;

#[cfg(feature = "metrics")]
use crate::metrics;

#[cfg(feature = "basic-auth")]
use crate::basic_auth;

#[cfg(feature = "mem-cache")]
use crate::mem_cache;

#[cfg(any(
    feature = "compression",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
))]
use crate::compression;

/// Output of the handler options initialization.
pub(super) struct HandlerOptsResult {
    /// Fully initialized request handler options.
    pub handler_opts: RequestHandlerOpts,
    /// Resolved 404 error page path (needed by the HTTPS redirect server).
    #[cfg(feature = "tls")]
    pub page404: std::path::PathBuf,
    /// Resolved 50x error page path (needed by the HTTPS redirect server).
    #[cfg(feature = "tls")]
    pub page50x: std::path::PathBuf,
}

/// Build and initialize `RequestHandlerOpts` from the given general and advanced settings.
///
/// This includes path validation, index-file parsing, and calling every
/// feature module's `init` function.
pub(super) fn init(general: &General, advanced: Option<Advanced>) -> Result<HandlerOptsResult> {
    // Validate root directory
    let root_dir = helpers::get_valid_dirpath(&general.root)
        .with_context(|| "root directory was not found or inaccessible")?;

    // Canonicalize the root directory once at startup
    // so further checks can compare against a precomputed canonical base
    // without paying a `canonicalize` syscall on every request.
    // Falls back to the validated path if canonicalization fails.
    // NOTE: When `use_relative_root` is enabled, canonicalization is skipped
    // so that symlinked root directories are resolved at request time.
    let root_dir = if general.use_relative_root {
        root_dir
    } else {
        root_dir.canonicalize().unwrap_or(root_dir)
    };

    // Resolve the 404 error page path relative to root when needed
    let mut page404 = general.page404.clone();
    if page404.is_relative() && !page404.starts_with(&root_dir) {
        page404 = root_dir.join(&page404);
    }
    if !page404.is_file() {
        tracing::debug!(
            "404 file path not found or not a regular file: {}",
            page404.display()
        );
    }

    // Resolve the 50x error page path relative to root when needed
    let mut page50x = general.page50x.clone();
    if page50x.is_relative() && !page50x.starts_with(&root_dir) {
        page50x = root_dir.join(&page50x);
    }
    if !page50x.is_file() {
        tracing::debug!(
            "50x file path not found or not a regular file: {}",
            page50x.display()
        );
    }

    tracing::info!(
        enabled = general.redirect_trailing_slash,
        "redirect trailing slash"
    );
    tracing::info!(enabled = general.include_hidden, "include hidden files");
    tracing::info!(enabled = general.follow_symlinks, "follow symlinks");
    tracing::info!(enabled = general.use_relative_root, "use relative root");

    // Default charset for text/* responses
    let default_text_charset = general.text_charset;
    tracing::info!(enabled = default_text_charset, "text charset");

    // Parse comma-separated index file list
    let index_files = general
        .index_files
        .split(',')
        .map(|s| s.trim().to_owned())
        .collect::<Vec<_>>();
    if index_files.is_empty() {
        bail!("index files list is empty, provide at least one index file")
    }
    tracing::info!(index_files = %general.index_files, "index files");

    let mut handler_opts = RequestHandlerOpts {
        root_dir,
        page404: page404.clone(),
        page50x: page50x.clone(),
        log_remote_address: general.log_remote_address,
        log_x_real_ip: general.log_x_real_ip,
        log_forwarded_for: general.log_forwarded_for,
        trusted_proxies: general.trusted_proxies.clone(),
        redirect_trailing_slash: general.redirect_trailing_slash,
        include_hidden: general.include_hidden,
        follow_symlinks: general.follow_symlinks,
        use_relative_root: general.use_relative_root,
        accept_markdown: general.accept_markdown,
        text_charset: general.text_charset,
        index_files,
        advanced_opts: advanced,
        ..Default::default()
    };

    // Directory listing
    #[cfg(feature = "directory-listing")]
    directory_listing::init(
        general.directory_listing,
        general.directory_listing_order,
        general.directory_listing_format.clone(),
        &mut handler_opts,
    );

    // Directory listing download
    #[cfg(feature = "directory-listing-download")]
    download::init(&general.directory_listing_download, &mut handler_opts);

    // Fallback page
    #[cfg(feature = "fallback-page")]
    fallback_page::init(&general.page_fallback, &mut handler_opts);

    // Pre-cache custom 404/50x bodies so error responses never touch disk
    // on the async hot path. See `error_page::PAGE_CACHE`.
    crate::error_page::cache_page(&handler_opts.page404);
    crate::error_page::cache_page(&handler_opts.page50x);

    // Health endpoint
    health::init(general.health, &mut handler_opts);

    // Log remote address
    log_addr::init(general.log_remote_address, &mut handler_opts);

    // Metrics endpoint
    #[cfg(feature = "metrics")]
    metrics::init(general.metrics, &mut handler_opts);

    // CORS
    cors::init(
        &general.cors_allow_origins,
        &general.cors_allow_headers,
        &general.cors_expose_headers,
        &mut handler_opts,
    );

    // Basic HTTP Authentication
    #[cfg(feature = "basic-auth")]
    basic_auth::init(&general.basic_auth, &mut handler_opts);

    // Maintenance mode
    maintenance_mode::init(
        general.maintenance_mode,
        general.maintenance_mode_status,
        general.maintenance_mode_file.clone(),
        &mut handler_opts,
    );

    // Pre-compressed static files
    compression_static::init(general.compression_static, &mut handler_opts);

    // Auto-compression based on Accept-Encoding header
    #[cfg(any(
        feature = "compression",
        feature = "compression-deflate",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd",
    ))]
    compression::init(
        general.compression,
        general.compression_level,
        &mut handler_opts,
    );

    // Cache-Control headers
    control_headers::init(general.cache_control_headers, &mut handler_opts);

    // Weak ETag headers
    etag::init(general.etag, &mut handler_opts);

    // Security headers
    security_headers::init(general.security_headers, &mut handler_opts);

    // In-memory cache
    #[cfg(feature = "mem-cache")]
    mem_cache::cache::init(&mut handler_opts)?;

    Ok(HandlerOptsResult {
        handler_opts,
        #[cfg(feature = "tls")]
        page404,
        #[cfg(feature = "tls")]
        page50x,
    })
}
