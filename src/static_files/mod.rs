// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The static file module which powers the web server.
//!
//! The request pipeline is intentionally linear and reads top-to-bottom in
//! [`handle`]:
//!
//! 1. **Method check** — `GET`, `HEAD` and `OPTIONS` only.
//! 2. **Path sanitization** — strip traversal components from the URI path.
//! 3. **In-memory cache lookup** — short-circuit hot files (experimental).
//! 4. **File resolution** — directory → index, `.html` fallback,
//!    pre-compressed variant detection (see [`resolve`]).
//! 5. **Security checks** — containment, symlink and hidden-file policy
//!    (see [`security`]).
//! 6. **Short-circuit responses** — trailing-slash redirect, `OPTIONS`,
//!    directory listing or archive download.
//! 7. **File reply** — stream the resolved file or its pre-compressed
//!    variant (see [`reply`]).

// Part of the module is borrowed and adapted at a convenience from
// https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

mod opts;
mod reply;
mod resolve;
mod security;

#[cfg(feature = "directory-listing")]
mod listing;

pub use opts::{HandleOpts, StaticFileResponse};

use hyper::StatusCode;

use crate::Result;
use crate::fs::meta::FileMetadata;
use crate::fs::path::sanitize_path;
use crate::http_ext::MethodExt;

#[cfg(feature = "experimental")]
use crate::mem_cache::cache;

/// The server entry point to handle incoming requests which map to specific files
/// on file system and return a file response.
pub async fn handle(opts: &HandleOpts<'_>) -> Result<StaticFileResponse, StatusCode> {
    if !opts.method.is_allowed() {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let mut file_path = sanitize_path(opts.base_path, opts.uri_path)?;

    // In-memory file cache lookup (experimental). A hit short-circuits the
    // pipeline; a miss returns a permit that lives until the file stream
    // finishes populating the cache.
    #[cfg(feature = "experimental")]
    let _cache_permit = match try_memory_cache(opts, &mut file_path).await? {
        MemCacheOutcome::Hit(resp) => return Ok(resp),
        MemCacheOutcome::Miss(permit) => permit,
    };

    let FileMetadata {
        file_path,
        metadata,
        is_dir,
        precompressed_variant,
    } = resolve::file_metadata(
        &mut file_path,
        opts.headers,
        opts.compression_static,
        opts.index_files,
    )?;

    security::enforce(file_path, is_dir, opts)?;

    let resp_file_path = file_path.to_owned();

    if let Some(resp) = reply::trailing_slash_redirect(is_dir, opts)? {
        return Ok(StaticFileResponse::new(resp, resp_file_path));
    }

    if opts.method.is_options() {
        return Ok(StaticFileResponse::new(
            reply::options_reply(),
            resp_file_path,
        ));
    }

    #[cfg(feature = "directory-listing")]
    if let Some(resp) = listing::try_listing(file_path, is_dir, opts)? {
        return Ok(StaticFileResponse::new(resp, resp_file_path));
    }

    let resp = reply::file_or_precompressed(opts, file_path, &metadata, precompressed_variant)?;
    Ok(StaticFileResponse::new(resp, resp_file_path))
}

/// Outcome of the experimental in-memory cache lookup.
#[cfg(feature = "experimental")]
enum MemCacheOutcome {
    /// The response is fully resolved from cache.
    Hit(StaticFileResponse),
    /// Cache miss. The held permit must be kept alive until the
    /// downstream file stream finishes inserting the file into the cache.
    Miss(Option<tokio::sync::SemaphorePermit<'static>>),
}

/// Tries to satisfy the request from the in-memory cache.
///
/// When a memory cache is configured and the request targets a directory
/// with trailing-slash redirect on, the cache key is the implicit
/// `<dir>/index.html`. The function may push that segment onto
/// `file_path` before performing the lookup.
#[cfg(feature = "experimental")]
async fn try_memory_cache(
    opts: &HandleOpts<'_>,
    file_path: &mut std::path::PathBuf,
) -> Result<MemCacheOutcome, StatusCode> {
    if opts.memory_cache.is_none() {
        return Ok(MemCacheOutcome::Miss(None));
    }

    // NOTE: only the default auto-index is supported for directory
    // requests inside the memory-cache context.
    if opts.redirect_trailing_slash && opts.uri_path.ends_with('/') {
        file_path.push("index.html");
    }

    let Some(result) = cache::get_or_acquire(file_path.as_path(), opts.headers).await else {
        return Ok(MemCacheOutcome::Miss(None));
    };

    match result {
        cache::CacheResult::Hit(result) => Ok(MemCacheOutcome::Hit(StaticFileResponse::new(
            result?,
            file_path.clone(),
        ))),
        cache::CacheResult::Error(status) => Err(status),
        cache::CacheResult::Miss(permit) => Ok(MemCacheOutcome::Miss(Some(permit))),
    }
}
