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
//! 3. **In-memory cache lookup** — short-circuit hot files.
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

// Re-export for benches/fuzzers/external integration tests that exercise
// the path-sanitisation invariant directly.
#[doc(hidden)]
pub use crate::fs::path::sanitize_path;

use hyper::StatusCode;

use crate::Result;
use crate::exts::http::MethodExt;
use crate::fs::meta::FileMetadata;

#[cfg(feature = "mem-cache")]
use crate::mem_cache::cache;

/// The server entry point to handle incoming requests which map to specific files
/// on file system and return a file response.
pub async fn handle(opts: &HandleOpts<'_>) -> Result<StaticFileResponse, StatusCode> {
    if !opts.method.is_allowed() {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let mut file_path = sanitize_path(opts.base_path, opts.uri_path)?;

    // In-memory file cache lookup. A hit short-circuits the pipeline.
    // On miss, the file is read from disk and the streaming pipeline
    // populates the cache opportunistically (see `mem_cache::stream`).
    #[cfg(feature = "mem-cache")]
    if let Some(resp) = try_memory_cache(opts, &mut file_path) {
        return Ok(resp);
    }

    let FileMetadata {
        file_path,
        metadata,
        is_dir,
        precompressed_variant,
        file,
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

    let resp =
        reply::file_or_precompressed(opts, file_path, &metadata, precompressed_variant, file)?;
    Ok(StaticFileResponse::new(resp, resp_file_path))
}

/// Tries to satisfy the request from the in-memory cache.
///
/// Returns `Some(response)` on a cache hit, `None` otherwise (cache disabled
/// in the runtime config, no entry yet, or a non-UTF-8 path).
///
/// When a memory cache is configured and the request targets a directory
/// with trailing-slash redirect on, the cache key is the implicit
/// `<dir>/index.html`. The function may push that segment onto
/// `file_path` before performing the lookup.
#[cfg(feature = "mem-cache")]
fn try_memory_cache(
    opts: &HandleOpts<'_>,
    file_path: &mut std::path::PathBuf,
) -> Option<StaticFileResponse> {
    // Runtime gate: if `[advanced.memory-cache]` is not configured in TOML,
    // `opts.memory_cache` is `None` and we skip the lookup entirely.
    opts.memory_cache.as_ref()?;

    // NOTE: only the default auto-index is supported for directory
    // requests inside the memory-cache context.
    if opts.redirect_trailing_slash && opts.uri_path.ends_with('/') {
        file_path.push("index.html");
    }

    let result = cache::lookup(file_path.as_path(), opts.headers)?;
    match result {
        Ok(resp) => Some(StaticFileResponse::new(resp, file_path.clone())),
        // Hit, but the cached entry returned an error status (e.g. malformed Range).
        // Fall through to the regular pipeline so the error path is consistent.
        Err(_) => None,
    }
}
