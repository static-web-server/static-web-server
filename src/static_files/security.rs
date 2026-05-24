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
//! 2. **Symlink policy** — when `--follow-symlinks` is off (the default),
//!    no path component may be a symlink.
//! 3. **Hidden file policy** — when `--include-hidden` is off (the default),
//!    dotfiles are reported as `404 Not Found`.
//!
//! The cheap string-only hidden check is intentionally evaluated *after*
//! containment so a path that escapes the base is rejected first, but
//! *before* file open so we never touch hidden files on disk.

use hyper::StatusCode;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::fs::path::PathExt;

use super::opts::HandleOpts;

/// Maximum number of `enforce_containment` "OK" decisions cached per
/// worker thread. Sized for typical static-file workloads where the
/// distinct request paths are small. When the cap is reached the
/// cache is dropped wholesale; the next requests pay the
/// `canonicalize` syscall again.
const CONTAINMENT_CACHE_CAP: usize = 1024;

/// Maximum age of a cached containment decision before it is evicted.
///
/// SECURITY: The containment cache stores positive ("safe") decisions
/// based on a `canonicalize` performed at insertion time. If an admin
/// replaces a previously-safe directory with a symlink at runtime, the
/// pre-existing cache entry would otherwise let the now-unsafe path
/// through. Periodic wholesale eviction caps that exposure window to
/// at most `CONTAINMENT_CACHE_TTL` seconds without measurable hot-path
/// cost (one `Instant::elapsed()` per insertion). The eviction is
/// deliberately coarse \u2014 finer-grained stat-based invalidation would
/// add a syscall to the fast path, defeating the cache's purpose.
const CONTAINMENT_CACHE_TTL: Duration = Duration::from_secs(60);

thread_local! {
    /// Per-thread set of `probe` paths that have previously been proven
    /// to live inside the canonical base directory.
    ///
    /// Profiling showed `enforce_containment` (and its
    /// `Path::canonicalize` syscall) was the single largest CPU cost on
    /// the static-file fast path \u2014 ~18% inclusive samples even after
    /// canonicalizing the base directory at startup. A workload that
    /// repeatedly serves the same documents reaches a steady state with
    /// effectively no `canonicalize` syscalls. The cache is keyed by
    /// `PathBuf` so the lookup is a single hash + byte compare.
    ///
    /// Cache validity: an entry is added only after the slow path has
    /// proven the probe is contained within `base_path`. The entire
    /// cache is dropped every `CONTAINMENT_CACHE_TTL` to bound the
    /// window in which a runtime filesystem mutation (e.g. an admin
    /// replacing a directory with a symlink) could yield a stale "OK".
    static CONTAINMENT_CACHE: RefCell<ContainmentCache> =
        RefCell::new(ContainmentCache::new());
}

/// Per-thread containment cache with TTL-based wholesale eviction.
struct ContainmentCache {
    entries: HashSet<PathBuf>,
    last_clear: Instant,
}

impl ContainmentCache {
    fn new() -> Self {
        Self {
            entries: HashSet::with_capacity(64),
            last_clear: Instant::now(),
        }
    }

    /// Returns `true` if `probe` is in the cache AND the cache has not
    /// expired. A stale cache returns `false` here and is cleared on
    /// the next insertion so we never serve a containment decision
    /// older than `CONTAINMENT_CACHE_TTL`.
    fn contains(&self, probe: &Path) -> bool {
        if self.last_clear.elapsed() > CONTAINMENT_CACHE_TTL {
            return false;
        }
        self.entries.contains(probe)
    }

    fn insert(&mut self, probe: PathBuf) {
        // Wholesale eviction on TTL expiry or capacity overflow. Both
        // paths reset `last_clear` so the cache starts a fresh window.
        if self.last_clear.elapsed() > CONTAINMENT_CACHE_TTL
            || self.entries.len() >= CONTAINMENT_CACHE_CAP
        {
            self.entries.clear();
            self.last_clear = Instant::now();
        }
        self.entries.insert(probe);
    }
}

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

    if !opts.follow_symlinks {
        enforce_symlink_policy(relative, opts.base_path, file_path)?;
    }

    if !opts.include_hidden && relative.is_hidden() {
        tracing::trace!(
            "considering hidden file {} as not found",
            file_path.display()
        );
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(())
}

/// Canonicalizes the requested file path and ensures it lives inside
/// the base directory.
///
/// **Performance note:** Callers should pass an already-canonical
/// `base_path` whenever possible. SWS canonicalizes the configured root
/// directory once at startup (see `server::opts::init`) and the same
/// for each virtual-host root (see `settings`), so the fast path below
/// avoids a `canonicalize` syscall on every request.
/// The function falls back to canonicalizing it, preserving the previous behavior.
fn enforce_containment(probe: &Path, base_path: &Path) -> Result<(), StatusCode> {
    // Fast path: the probe was already proven safe on a previous request
    // (see `CONTAINMENT_CACHE`). Skips the per-request `canonicalize`
    // syscall entirely for the common repeat-hit case.
    if CONTAINMENT_CACHE.with(|c| c.borrow().contains(probe)) {
        return Ok(());
    }

    let file_path_resolved = probe.canonicalize().map_err(|err| {
        tracing::error!(
            "unable to resolve '{}' symlink path: {}",
            probe.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    // a. Fast path: when `base_path` is already canonical (the production case),
    // the resolved file path will share its prefix and we avoid a per-request syscall.
    if file_path_resolved.starts_with(base_path) {
        cache_safe_probe(probe);
        return Ok(());
    }

    // b. Fallback: canonicalize the base and retry the check.
    // Keeps the function correct for callers that pass non-canonical paths.
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

    cache_safe_probe(probe);
    Ok(())
}

/// Records `probe` as previously-verified-safe in the per-thread
/// containment cache. When the cache fills or its TTL expires the entire
/// set is dropped rather than performing per-entry LRU/expiry bookkeeping,
/// since the working set is expected to fit well within
/// `CONTAINMENT_CACHE_CAP` and the eviction is amortized.
#[inline]
fn cache_safe_probe(probe: &Path) {
    CONTAINMENT_CACHE.with(|c| {
        c.borrow_mut().insert(probe.to_path_buf());
    });
}

/// Walks each component of `relative` and rejects the request if any
/// of them is a symlink.
///
/// This is a syscall-per-component check, so callers should gate it
/// behind the `--follow-symlinks` flag.
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
