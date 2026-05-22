// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Security checks applied to a resolved file path before it is served.
//!
//! This module enforces SWS's multi-layer path-traversal defense:
//!
//! 1. **Containment** — the canonical resolved path must live inside the
//!    canonical base directory.
//! 2. **Symlink policy** — when `--disable-symlinks` is on, no path component
//!    may be a symlink.
//! 3. **Hidden file policy** — when `--ignore-hidden-files` is on, dotfiles
//!    are reported as `404 Not Found`.
//!
//! The cheap string-only hidden check is intentionally evaluated *after*
//! containment so a path that escapes the base is rejected first, but
//! *before* file open so we never touch hidden files on disk.

use hyper::StatusCode;
use std::path::Path;

use crate::fs::path::PathExt;

use super::opts::HandleOpts;

/// Verifies that `file_path` is safe to serve under the current `opts`.
///
/// Returns `Ok(())` when all security checks pass. Returns a `StatusCode`
/// describing how the request should be denied otherwise.
pub(super) fn enforce(
    file_path: &Path,
    is_dir: bool,
    opts: &HandleOpts<'_>,
) -> Result<(), StatusCode> {
    // For directory requests, the metadata-resolved `file_path` already
    // points at the index file. Strip that segment so containment and
    // symlink checks reflect the directory the user requested.
    let mut probe = file_path.to_path_buf();
    if is_dir {
        probe.pop();
    }

    let relative = probe.strip_prefix(opts.base_path).map_err(|err| {
        tracing::error!(
            "unable to strip prefix from file path '{}': {}",
            file_path.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    enforce_containment(&probe, opts.base_path)?;

    if opts.disable_symlinks {
        enforce_symlink_policy(relative, opts.base_path, file_path)?;
    }

    if opts.ignore_hidden_files && relative.is_hidden() {
        tracing::trace!(
            "considering hidden file {} as not found",
            file_path.display()
        );
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(())
}

/// Canonicalizes both paths and ensures the resolved file lives inside
/// the resolved base directory.
fn enforce_containment(probe: &Path, base_path: &Path) -> Result<(), StatusCode> {
    let file_path_resolved = probe.canonicalize().map_err(|err| {
        tracing::error!(
            "unable to resolve '{}' symlink path: {}",
            probe.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    let base_path_resolved = base_path.canonicalize().map_err(|err| {
        tracing::error!(
            "unable to resolve '{}' base path: {}",
            base_path.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    if !file_path_resolved.starts_with(base_path_resolved) {
        tracing::error!(
            "file path '{}' resolves outside of the base path, access denied",
            file_path_resolved.display()
        );
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(())
}

/// Walks each component of `relative` and rejects the request if any
/// of them is a symlink.
///
/// This is a syscall-per-component check, so callers should gate it
/// behind the `--disable-symlinks` flag.
fn enforce_symlink_policy(
    relative: &Path,
    base_path: &Path,
    file_path: &Path,
) -> Result<(), StatusCode> {
    let has_symlink = relative.contains_symlink(base_path).map_err(|err| {
        tracing::error!(
            "unable to check if file path '{}' contains symlink: {}",
            relative.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    if has_symlink {
        tracing::warn!(
            "file path '{}' contains a symlink, access denied",
            file_path.display()
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(())
}
