// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The static file module which powers the web server.
//!

// Part of the file is borrowed and adapted at a convenience from
// https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

use headers::{AcceptRanges, HeaderMap, HeaderMapExt, HeaderValue};
use hyper::{Body, Method, Response, StatusCode, header::CONTENT_ENCODING, header::CONTENT_LENGTH};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::{File, Metadata};
use std::path::{Path, PathBuf};

use crate::Result;
use crate::conditional_headers::ConditionalHeaders;
use crate::fs::meta::{FileMetadata, try_file_open, try_metadata, try_metadata_with_html_suffix};
use crate::fs::path::{PathExt, sanitize_path};
use crate::http_ext::{HTTP_SUPPORTED_METHODS, MethodExt};
use crate::response::response_body;

#[cfg(feature = "experimental")]
use crate::mem_cache::{cache, cache::MemCacheOpts};

use crate::compression_static;

#[cfg(feature = "directory-listing")]
use crate::{
    directory_listing,
    directory_listing::{DirListFmt, DirListOpts},
};

#[cfg(feature = "directory-listing-download")]
use crate::directory_listing_download::{
    DOWNLOAD_PARAM_KEY, DirDownloadFmt, DirDownloadOpts, archive_reply,
};

const DEFAULT_INDEX_FILES: &[&str; 1] = &["index.html"];

/// Maximum number of containment "OK" decisions cached per worker thread.
/// Sized for typical static-file workloads where the distinct request paths
/// are small. When the cap is reached the cache is dropped wholesale; the
/// next requests pay the `canonicalize` syscall again.
const CONTAINMENT_CACHE_CAP: usize = 1024;

#[cfg(test)]
thread_local! {
    /// Runs once after the selected response file has been opened, at the point
    /// where the old response path would open it again by pathname.
    static FILE_OPEN_RACE_HOOK: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);

    /// Runs once after the containment and symlink policies have been enforced
    /// but *before* the selected response file is opened. Used to simulate an
    /// attacker retargeting a symlink inside the check-then-open window.
    static PRE_FILE_OPEN_RACE_HOOK: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);
}

thread_local! {
    /// Per-thread set of `probe` paths that have previously been proven
    /// to live inside the canonical base directory.
    ///
    /// Profiling showed the containment check (and its `Path::canonicalize`
    /// syscall) was the single largest CPU cost on the static-file fast
    /// path. A workload that repeatedly serves the same documents reaches
    /// a steady state with effectively no `canonicalize` syscalls. The
    /// cache is keyed by `PathBuf` so the lookup is a single hash + byte
    /// compare.
    ///
    /// Cache validity: an entry is added only after the slow path has
    /// proven the probe is contained within `base_path`. Callers only
    /// reuse entries while enforcing the no-symlink policy on every request.
    static CONTAINMENT_CACHE: RefCell<HashSet<PathBuf>> =
        RefCell::new(HashSet::with_capacity(64));
}

#[cfg(test)]
fn set_file_open_race_hook(hook: impl FnOnce() + 'static) {
    FILE_OPEN_RACE_HOOK.with(|slot| *slot.borrow_mut() = Some(Box::new(hook)));
}

#[cfg(test)]
fn run_file_open_race_hook() {
    FILE_OPEN_RACE_HOOK.with(|slot| {
        if let Some(hook) = slot.borrow_mut().take() {
            hook();
        }
    });
}

#[cfg(test)]
fn set_pre_file_open_race_hook(hook: impl FnOnce() + 'static) {
    PRE_FILE_OPEN_RACE_HOOK.with(|slot| *slot.borrow_mut() = Some(Box::new(hook)));
}

#[cfg(test)]
fn run_pre_file_open_race_hook() {
    PRE_FILE_OPEN_RACE_HOOK.with(|slot| {
        if let Some(hook) = slot.borrow_mut().take() {
            hook();
        }
    });
}

#[cfg(test)]
thread_local! {
    /// Counts `Path::canonicalize` calls issued by [`enforce_containment`] on
    /// the current thread. Used by the tests to assert that the positive
    /// containment cache is bypassed whenever symlinks are followed.
    static CANONICALIZE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn reset_canonicalize_calls() {
    CANONICALIZE_CALLS.with(|counter| counter.set(0));
}

#[cfg(test)]
fn canonicalize_calls() -> usize {
    CANONICALIZE_CALLS.with(std::cell::Cell::get)
}

/// Records `probe` as previously-verified-safe in the per-thread
/// containment cache. When the cache fills, the entire set is dropped
/// rather than performing per-entry LRU bookkeeping, since the working
/// set is expected to fit well within `CONTAINMENT_CACHE_CAP`.
#[inline]
fn cache_safe_probe(probe: &Path) {
    CONTAINMENT_CACHE.with(|c| {
        let mut set = c.borrow_mut();
        if set.len() >= CONTAINMENT_CACHE_CAP {
            set.clear();
        }
        set.insert(probe.to_path_buf());
    });
}

/// Defines all options needed by the static-files handler.
pub struct HandleOpts<'a> {
    /// Request method.
    pub method: &'a Method,
    /// In-memory files cache feature (experimental).
    #[cfg(feature = "experimental")]
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
    /// Ignore hidden files feature.
    pub ignore_hidden_files: bool,
    /// Prevent following symlinks for files and directories.
    pub disable_symlinks: bool,
}

/// The path that is safe to open once the security checks have passed.
///
/// Opening the *canonical* path rather than the originally requested one
/// narrows the check-then-open (TOCTOU) window: a canonical path contains no
/// symlink components, so a symlink retargeted between the containment check
/// and the `open(2)` can no longer redirect the read outside the web root.
enum SafeOpenPath {
    /// A fully-resolved, symlink-free path that is safe to open directly.
    Canonical(PathBuf),
    /// No canonical path is available, so the caller must open the requested
    /// path. This is returned when either:
    /// * the containment decision was reused from the per-thread cache, which
    ///   only happens while the no-symlink policy re-validates every path
    ///   component on every request, or
    /// * the checked path is a directory-listing placeholder that is never
    ///   opened as a file.
    Requested,
}

impl SafeOpenPath {
    /// Returns the path to open, falling back to `requested` when no canonical
    /// path was resolved.
    #[inline]
    fn as_path<'a>(&'a self, requested: &'a Path) -> &'a Path {
        match self {
            Self::Canonical(path) => path.as_path(),
            Self::Requested => requested,
        }
    }
}

/// Verifies that `file_path` is safe to serve under the current `opts` and
/// returns the path that is safe to open.
///
/// `file_path` must be the path that will actually be opened and streamed. For
/// directory requests that resolved an index file this is the index file
/// itself, so a symlinked index file is covered by the containment and symlink
/// checks as well. When a directory request did *not* resolve an index file
/// (`resolved_exists == false`) the trailing segment is a directory-listing
/// placeholder that does not exist on disk, so it is stripped before checking
/// (canonicalizing a non-existent path always fails).
fn enforce_path_security(
    file_path: &Path,
    is_dir: bool,
    resolved_exists: bool,
    opts: &HandleOpts<'_>,
    use_containment_cache: bool,
) -> Result<SafeOpenPath, StatusCode> {
    let is_placeholder = is_dir && !resolved_exists;
    let mut probe = file_path.to_path_buf();
    if is_placeholder {
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

    let contained = enforce_containment(&probe, opts.base_path, use_containment_cache)?;

    if opts.disable_symlinks {
        enforce_symlink_policy(relative, opts.base_path, file_path)?;
    }

    // Check for a hidden file/directory (dotfile) and ignore it if feature enabled.
    // The appended index-file segment of a directory request is server-configured
    // rather than client-supplied, so it is excluded from this check.
    let hidden_relative = if is_dir && resolved_exists {
        relative.parent().unwrap_or(relative)
    } else {
        relative
    };
    if opts.ignore_hidden_files && hidden_relative.is_hidden() {
        tracing::trace!(
            "considering hidden file {} as not found",
            file_path.display()
        );
        return Err(StatusCode::NOT_FOUND);
    }

    // A placeholder path is only used to build a directory listing, never
    // opened, and its canonical form points at the parent directory. Never hand
    // it back as an open target.
    if is_placeholder {
        return Ok(SafeOpenPath::Requested);
    }

    Ok(match contained {
        Some(resolved) => SafeOpenPath::Canonical(resolved),
        None => SafeOpenPath::Requested,
    })
}

/// Canonicalizes `probe` and verifies that it resolves within `base_path`.
///
/// Returns the resolved (symlink-free) path on success, or `None` when the
/// decision was reused from the per-thread containment cache.
///
/// `use_cache` must only be enabled when the caller independently checks every
/// path component for symlinks on each request. Otherwise a symlink retargeted
/// after a cached decision could resolve outside `base_path`.
fn enforce_containment(
    probe: &Path,
    base_path: &Path,
    use_cache: bool,
) -> Result<Option<PathBuf>, StatusCode> {
    if use_cache && CONTAINMENT_CACHE.with(|cache| cache.borrow().contains(probe)) {
        return Ok(None);
    }

    #[cfg(test)]
    CANONICALIZE_CALLS.with(|counter| counter.set(counter.get() + 1));

    let resolved = probe.canonicalize().map_err(|err| {
        tracing::error!(
            "unable to resolve '{}' symlink path: {}",
            probe.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    // Fast path: when `base_path` is already canonical (the production case),
    // the resolved file path will share its prefix and we avoid canonicalizing
    // the base directory again.
    if resolved.starts_with(base_path) {
        if use_cache {
            cache_safe_probe(probe);
        }
        return Ok(Some(resolved));
    }

    // Fallback for callers that provide a non-canonical base path.
    let base_path = base_path.canonicalize().map_err(|err| {
        tracing::error!(
            "unable to resolve '{}' base path: {}",
            base_path.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    if !resolved.starts_with(&base_path) {
        tracing::error!(
            "file path '{}' resolves outside of the base path, access denied",
            resolved.display()
        );
        return Err(StatusCode::NOT_FOUND);
    }

    if use_cache {
        cache_safe_probe(probe);
    }
    Ok(Some(resolved))
}

/// Rejects `file_path` when it or any relative path component is a symlink.
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

/// Static file response type with additional data.
pub struct StaticFileResponse {
    /// Inner HTTP response.
    pub resp: Response<Body>,
    /// The file path of the inner HTTP response.
    pub file_path: PathBuf,
}

/// The server entry point to handle incoming requests which map to specific files
/// on file system and return a file response.
pub async fn handle(opts: &HandleOpts<'_>) -> Result<StaticFileResponse, StatusCode> {
    let method = opts.method;
    // Check if current HTTP method for incoming request is supported
    if !method.is_allowed() {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let uri_path = opts.uri_path;
    let mut file_path = sanitize_path(opts.base_path, uri_path)?;

    let headers_opt = opts.headers;

    // In-memory file cache feature with eviction policy
    #[cfg(feature = "experimental")]
    if opts.memory_cache.is_some() {
        // NOTE: we only support a default auto index for directory requests
        // when working on a memory-cache context.
        if opts.redirect_trailing_slash && uri_path.ends_with('/') {
            file_path.push("index.html");
        }

        if let Some(result) = cache::get_or_acquire(file_path.as_path(), headers_opt).await {
            match result {
                cache::CacheResult::Hit(result) => {
                    return Ok(StaticFileResponse {
                        resp: result?,
                        file_path,
                    });
                }
                cache::CacheResult::Error(status) => {
                    return Err(status);
                }
                cache::CacheResult::Miss(_permit) => {
                    // Permit is held while we proceed to read the file below.
                    // It will be dropped at the end of this scope, after the
                    // MemCacheFileStream inserts the data into the cache store.
                }
            }
        }
    }

    let FileMetadata {
        file_path,
        is_dir,
        resolved_exists,
        precompressed_variant,
    } = get_composed_file_metadata(
        &mut file_path,
        headers_opt,
        opts.compression_static,
        opts.index_files,
    )?;

    // The positive containment cache may only be reused when the no-symlink
    // policy re-validates every path component on every request. When symlinks
    // are allowed, a symlink retargeted after a cached decision could resolve
    // outside the web root, so the cache is bypassed and each request pays the
    // `canonicalize` syscall again. This trades throughput for containment
    // correctness on the default (symlink-following) configuration.
    let use_containment_cache = opts.disable_symlinks;

    let safe_path = enforce_path_security(
        file_path,
        is_dir,
        resolved_exists,
        opts,
        use_containment_cache,
    )?;

    // Variant selection changes the file that will be opened, so the selected
    // pre-compressed path must pass the very same containment and symlink
    // checks as the originally requested path before it can be served.
    let safe_precompressed_path = match precompressed_variant.as_ref() {
        Some((precompressed_path, _)) => Some(enforce_path_security(
            precompressed_path,
            false,
            true,
            opts,
            use_containment_cache,
        )?),
        None => None,
    };

    let resp_file_path = file_path.to_owned();

    // Check for a trailing slash on the current directory path
    // and redirect if that path doesn't end with the slash char
    if is_dir && opts.redirect_trailing_slash && !uri_path.ends_with('/') {
        let query = opts.uri_query.map_or(String::new(), |s| ["?", s].concat());
        let uri = [uri_path, "/", query.as_str()].concat();
        let loc = match HeaderValue::from_str(uri.as_str()) {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("invalid header value from current uri: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut resp = Response::new(Body::empty());
        resp.headers_mut().insert(hyper::header::LOCATION, loc);
        *resp.status_mut() = StatusCode::PERMANENT_REDIRECT;

        tracing::trace!("uri doesn't end with a slash so redirecting permanently");
        return Ok(StaticFileResponse {
            resp,
            file_path: resp_file_path,
        });
    }

    // Respond with the permitted communication methods
    if method.is_options() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::NO_CONTENT;
        resp.headers_mut()
            .typed_insert(headers::Allow::from_iter(HTTP_SUPPORTED_METHODS.clone()));
        resp.headers_mut().typed_insert(AcceptRanges::bytes());

        return Ok(StaticFileResponse {
            resp,
            file_path: resp_file_path,
        });
    }

    // Directory listing
    // Check if "directory listing" feature is enabled,
    // if current path is a valid directory and
    // if it does not contain an `index.html` file (if a proper auto index is generated)
    #[cfg(feature = "directory-listing")]
    if is_dir && opts.dir_listing && !resolved_exists {
        // Directory listing download
        // Check if "directory listing download" feature is enabled,
        // if current path is a valid directory and
        // if query string has parameter "download" set
        #[cfg(feature = "directory-listing-download")]
        if !opts.dir_listing_download.is_empty()
            && let Some((_k, _dl_archive_opt)) =
                form_urlencoded::parse(opts.uri_query.unwrap_or("").as_bytes())
                    .find(|(k, _v)| k == DOWNLOAD_PARAM_KEY)
        {
            // file path is index.html, need pop
            let mut fp = file_path.clone();
            fp.pop();
            if let Some(filename) = fp.file_name() {
                let resp = archive_reply(
                    filename,
                    &fp,
                    DirDownloadOpts {
                        method,
                        disable_symlinks: opts.disable_symlinks,
                        ignore_hidden_files: opts.ignore_hidden_files,
                    },
                );
                return Ok(StaticFileResponse {
                    resp,
                    file_path: resp_file_path,
                });
            } else {
                tracing::error!("Unable to get filename from {}", fp.to_string_lossy());
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }

        let resp = directory_listing::auto_index(DirListOpts {
            root_path: opts.base_path.as_path(),
            method,
            current_path: uri_path,
            uri_query: opts.uri_query,
            filepath: file_path,
            dir_listing_order: opts.dir_listing_order,
            dir_listing_format: opts.dir_listing_format,
            ignore_hidden_files: opts.ignore_hidden_files,
            disable_symlinks: opts.disable_symlinks,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: opts.dir_listing_download,
        })?;

        return Ok(StaticFileResponse {
            resp,
            file_path: resp_file_path,
        });
    }

    // Check for a pre-compressed file variant if present under the `opts.compression_static` context
    if let Some((precomp_path, precomp_encoding)) = precompressed_variant {
        // Open the canonical (symlink-free) path resolved by the containment
        // check so a symlink retargeted between the check and this `open(2)`
        // cannot redirect the read outside the web root.
        let open_path = safe_precompressed_path
            .as_ref()
            .map_or(precomp_path.as_path(), |safe| safe.as_path(&precomp_path));

        #[cfg(test)]
        run_pre_file_open_race_hook();

        let (file, precomp_meta) = try_file_open(open_path)?;

        #[cfg(test)]
        run_file_open_race_hook();

        let mut resp = file_reply(
            headers_opt,
            file_path,
            &precomp_meta,
            file,
            // Never insert a pre-compressed body into the in-memory cache: the
            // cache is keyed by the *original* path and its lookup runs before
            // content negotiation, so a cached variant body would be served
            // (undecodable) to clients that did not accept that encoding.
            #[cfg(feature = "experimental")]
            None,
        )?;

        // Prepare corresponding headers to let know how to decode the payload
        resp.headers_mut().remove(CONTENT_LENGTH);
        let encoding = match HeaderValue::from_str(precomp_encoding.as_str()) {
            Ok(val) => val,
            Err(err) => {
                tracing::error!(
                    "unable to parse header value from content encoding: {:?}",
                    err
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        resp.headers_mut().insert(CONTENT_ENCODING, encoding);

        return Ok(StaticFileResponse {
            resp,
            file_path: resp_file_path,
        });
    }

    // Same rationale as the pre-compressed branch: open the canonical path
    // proven contained by the security checks above.
    #[cfg(test)]
    run_pre_file_open_race_hook();

    let (file, serve_meta) = try_file_open(safe_path.as_path(file_path))?;

    #[cfg(test)]
    run_file_open_race_hook();

    #[cfg(feature = "experimental")]
    let resp = file_reply(headers_opt, file_path, &serve_meta, file, opts.memory_cache)?;

    #[cfg(not(feature = "experimental"))]
    let resp = file_reply(headers_opt, file_path, &serve_meta, file)?;

    Ok(StaticFileResponse {
        resp,
        file_path: resp_file_path,
    })
}

/// Returns the final composed metadata containing
/// the current `file_path` with its file metadata
/// as well as its optional pre-compressed variant.
///
/// This resolver only *probes* the file system to determine which path should
/// be served. It deliberately never hands an open file handle to the caller:
/// the file that is streamed must be opened only after the containment and
/// symlink checks have run, and from the canonical path they resolved. See
/// [`enforce_path_security`].
fn get_composed_file_metadata<'a>(
    mut file_path: &'a mut PathBuf,
    headers: &'a HeaderMap<HeaderValue>,
    compression_static: bool,
    mut index_files: &'a [&'a str],
) -> Result<FileMetadata<'a>, StatusCode> {
    tracing::trace!("getting metadata for file {}", file_path.display());

    // Try to find the file path on the file system
    match try_metadata(file_path) {
        Ok((_, is_dir)) => {
            // Whether the resolved `file_path` points to an existing file.
            // For non-directory requests this is always true.
            // For directory requests it becomes true only when an index file (or its
            // `.html` suffix sibling) was successfully resolved. Used to
            // gate the pre-compressed variant probe so we never issue
            // `stat(2)` for `.br`/`.gz`/`.zst` siblings of a non-existent
            // index (see issue #617).
            let mut resolved_exists = !is_dir;
            if is_dir {
                // Try every index file variant in order
                if index_files.is_empty() {
                    index_files = DEFAULT_INDEX_FILES;
                }
                for index in index_files {
                    // Append a HTML index page by default if it's a directory path (`autoindex`)
                    tracing::debug!("dir: appending {} to the directory path", index);
                    file_path.push(index);

                    if matches!(try_metadata(file_path), Ok((_, false))) {
                        resolved_exists = true;
                        break;
                    }

                    // We remove only the appended index file
                    file_path.pop();
                    let new_meta: Option<Metadata>;
                    (file_path, new_meta) = try_metadata_with_html_suffix(file_path);
                    if new_meta.is_some() {
                        resolved_exists = true;
                        break;
                    }
                }

                // In case no index was found then we append the last index
                // of the list to preserve the previous behavior
                if !resolved_exists && !index_files.is_empty() {
                    file_path.push(index_files.last().unwrap());
                }
            }

            // Only probe for pre-compressed siblings when the resolved file
            // actually exists. Probing for `.br`/`.gz`/`.zst` of a path that
            // was never confirmed on disk wastes one `stat(2)` per
            // configured encoding on the request hot path
            // (see issue #617).
            let precompressed_variant = (compression_static && resolved_exists)
                .then(|| compression_static::precompressed_variant(file_path, headers))
                .flatten()
                .map(|p| (p.file_path, p.encoding));

            Ok(FileMetadata {
                file_path,
                is_dir,
                resolved_exists,
                precompressed_variant,
            })
        }
        Err(err) => {
            // If the file path doesn't exist, then try the `.html`-suffixed path
            // first. For example: `/posts/article` falls back to
            // `/posts/article.html`.
            //
            // We intentionally do *not* probe for pre-compressed siblings
            // of the original (non-existent) path. Doing so would waste
            // one `stat(2)` per configured encoding for every truly
            // missing path (see issue #617).
            let new_meta: Option<Metadata>;
            (file_path, new_meta) = try_metadata_with_html_suffix(file_path);

            if new_meta.is_none() {
                // Neither the original path nor its `.html` sibling exists.
                // Return the original error without probing for compressed
                // variants of non-existent files.
                return Err(err);
            }

            // The `.html` sibling exists. Only now is it worth probing for
            // its pre-compressed sibling (`/article.html.br`, etc.).
            let precompressed_variant = compression_static
                .then(|| compression_static::precompressed_variant(file_path, headers))
                .flatten()
                .map(|p| (p.file_path, p.encoding));

            Ok(FileMetadata {
                file_path,
                is_dir: false,
                resolved_exists: true,
                precompressed_variant,
            })
        }
    }
}

/// Builds a response from an already-opened and validated file handle.
fn file_reply<'a>(
    headers: &'a HeaderMap<HeaderValue>,
    path: &'a Path,
    meta: &'a Metadata,
    file: File,
    #[cfg(feature = "experimental")] memory_cache: Option<&'a MemCacheOpts>,
) -> Result<Response<Body>, StatusCode> {
    let conditionals = ConditionalHeaders::new(headers);

    #[cfg(feature = "experimental")]
    let resp = response_body(file, path, meta, conditionals, memory_cache);

    #[cfg(not(feature = "experimental"))]
    let resp = response_body(file, path, meta, conditionals);

    resp
}

#[cfg(all(test, unix))]
mod tests {
    use super::{
        HandleOpts, StaticFileResponse, canonicalize_calls, handle, reset_canonicalize_calls,
        set_file_open_race_hook, set_pre_file_open_race_hook,
    };
    use headers::HeaderMap;
    use hyper::{Method, StatusCode};
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let path = std::env::temp_dir().join(format!(
                "sws-static-files-{tag}-{}-{nanos}-{seq}",
                std::process::id(),
            ));
            fs::create_dir_all(&path).expect("unexpected error creating temporary directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Creates a temporary web root containing an `app.js` source file and
    /// returns the temporary directory guard alongside the web root path.
    fn web_root(tag: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new(tag);
        let root_dir = temp_dir.path().join("public");
        fs::create_dir(&root_dir).expect("unexpected error creating web root");
        (temp_dir, root_dir)
    }

    struct RequestOpts<'a> {
        uri_path: &'a str,
        encoding: &'a str,
        disable_symlinks: bool,
        ignore_hidden_files: bool,
        index_files: &'a [&'a str],
    }

    impl Default for RequestOpts<'_> {
        fn default() -> Self {
            Self {
                uri_path: "/app.js",
                encoding: "identity",
                disable_symlinks: false,
                ignore_hidden_files: false,
                index_files: &[],
            }
        }
    }

    async fn request_with(
        root_dir: &PathBuf,
        opts: RequestOpts<'_>,
    ) -> Result<StaticFileResponse, StatusCode> {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            opts.encoding.parse().expect("unexpected invalid encoding"),
        );

        handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: root_dir,
            uri_path: opts.uri_path,
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &crate::directory_listing::DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: true,
            ignore_hidden_files: opts.ignore_hidden_files,
            disable_symlinks: opts.disable_symlinks,
            index_files: opts.index_files,
        })
        .await
    }

    async fn request(
        root_dir: &PathBuf,
        encoding: &str,
        disable_symlinks: bool,
    ) -> Result<StaticFileResponse, StatusCode> {
        request_with(
            root_dir,
            RequestOpts {
                encoding,
                disable_symlinks,
                ..Default::default()
            },
        )
        .await
    }

    async fn body_of(result: StaticFileResponse) -> Vec<u8> {
        hyper::body::to_bytes(result.resp.into_body())
            .await
            .expect("unexpected bytes error during body conversion")
            .to_vec()
    }

    #[tokio::test]
    async fn opened_precompressed_file_retargeted_before_response_uses_opened_handle() {
        for (extension, encoding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            for disable_symlinks in [false, true] {
                let (temp_dir, root_dir) = web_root("open-race");
                fs::write(root_dir.join("app.js"), "source")
                    .expect("unexpected error writing source file");
                let inside_marker = root_dir.join("inside-marker");
                fs::write(&inside_marker, "inside")
                    .expect("unexpected error writing inside marker");
                let outside_marker = temp_dir.path().join("outside-marker");
                fs::write(&outside_marker, "beyond")
                    .expect("unexpected error writing outside marker");
                let precompressed_path = root_dir.join(format!("app.js.{extension}"));

                if disable_symlinks {
                    fs::write(&precompressed_path, "inside")
                        .expect("unexpected error writing pre-compressed file");
                } else {
                    symlink(&inside_marker, &precompressed_path)
                        .expect("unexpected error creating contained pre-compressed symlink");
                }

                let race_path = precompressed_path.clone();
                set_file_open_race_hook(move || {
                    fs::remove_file(&race_path)
                        .expect("unexpected error removing pre-compressed path");
                    symlink(&outside_marker, &race_path)
                        .expect("unexpected error retargeting pre-compressed path");
                });

                let result = request(&root_dir, encoding, disable_symlinks)
                    .await
                    .expect("expected pre-compressed response");

                assert_eq!(
                    body_of(result).await,
                    b"inside",
                    "served retargeted {encoding} path with disable_symlinks={disable_symlinks}"
                );
            }
        }
    }

    #[tokio::test]
    async fn precompressed_file_retargeted_within_check_open_window_is_not_served() {
        for (extension, encoding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            let (temp_dir, root_dir) = web_root("precomp-check-open-race");
            fs::write(root_dir.join("app.js"), "source")
                .expect("unexpected error writing source file");
            let inside_marker = root_dir.join("inside-marker");
            fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");
            let outside_marker = temp_dir.path().join("outside-marker");
            fs::write(&outside_marker, "beyond").expect("unexpected error writing outside marker");

            let precompressed_path = root_dir.join(format!("app.js.{extension}"));
            symlink(&inside_marker, &precompressed_path)
                .expect("unexpected error creating contained pre-compressed symlink");

            // Retarget the symlink outside the web root *after* the containment
            // check has passed but *before* the file is opened.
            let race_path = precompressed_path.clone();
            set_pre_file_open_race_hook(move || {
                fs::remove_file(&race_path).expect("unexpected error removing pre-compressed path");
                symlink(&outside_marker, &race_path)
                    .expect("unexpected error retargeting pre-compressed path");
            });

            let result = request(&root_dir, encoding, false)
                .await
                .expect("expected pre-compressed response");

            assert_eq!(
                body_of(result).await,
                b"inside",
                "served {encoding} variant retargeted inside the check-then-open window"
            );
        }
    }

    #[tokio::test]
    async fn requested_file_retargeted_within_check_open_window_is_not_served() {
        let (temp_dir, root_dir) = web_root("requested-check-open-race");
        let inside_marker = root_dir.join("inside-marker");
        fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");
        let outside_marker = temp_dir.path().join("outside-marker");
        fs::write(&outside_marker, "beyond").expect("unexpected error writing outside marker");

        let requested_path = root_dir.join("app.js");
        symlink(&inside_marker, &requested_path)
            .expect("unexpected error creating contained requested symlink");

        let race_path = requested_path.clone();
        set_pre_file_open_race_hook(move || {
            fs::remove_file(&race_path).expect("unexpected error removing requested path");
            symlink(&outside_marker, &race_path)
                .expect("unexpected error retargeting requested path");
        });

        let result = request(&root_dir, "identity", false)
            .await
            .expect("expected requested-file response");

        assert_eq!(
            body_of(result).await,
            b"inside",
            "served requested file retargeted inside the check-then-open window"
        );
    }

    #[tokio::test]
    async fn index_file_symlink_escaping_root_is_rejected() {
        let (temp_dir, root_dir) = web_root("index-symlink-outside");
        let outside_marker = temp_dir.path().join("outside-marker");
        fs::write(&outside_marker, "beyond").expect("unexpected error writing outside marker");
        symlink(&outside_marker, root_dir.join("index.html"))
            .expect("unexpected error creating escaping index symlink");

        match request_with(
            &root_dir,
            RequestOpts {
                uri_path: "/",
                ..Default::default()
            },
        )
        .await
        {
            Ok(result) => panic!(
                "expected escaping index symlink to be rejected, got {}",
                result.resp.status()
            ),
            Err(status) => assert_eq!(status, StatusCode::NOT_FOUND),
        }
    }

    #[tokio::test]
    async fn index_file_symlink_is_rejected_when_symlinks_are_disabled() {
        let (_temp_dir, root_dir) = web_root("index-symlink-disabled");
        let inside_marker = root_dir.join("inside-marker");
        fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");
        symlink(&inside_marker, root_dir.join("index.html"))
            .expect("unexpected error creating contained index symlink");

        match request_with(
            &root_dir,
            RequestOpts {
                uri_path: "/",
                disable_symlinks: true,
                ..Default::default()
            },
        )
        .await
        {
            Ok(result) => panic!(
                "expected index symlink to be rejected, got {}",
                result.resp.status()
            ),
            Err(status) => assert_eq!(status, StatusCode::FORBIDDEN),
        }
    }

    #[tokio::test]
    async fn index_file_symlink_inside_root_is_served_when_symlinks_are_enabled() {
        let (_temp_dir, root_dir) = web_root("index-symlink-allowed");
        let inside_marker = root_dir.join("inside-marker");
        fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");
        symlink(&inside_marker, root_dir.join("index.html"))
            .expect("unexpected error creating contained index symlink");

        let result = request_with(
            &root_dir,
            RequestOpts {
                uri_path: "/",
                ..Default::default()
            },
        )
        .await
        .expect("expected contained index symlink to be served");

        assert_eq!(body_of(result).await, b"inside");
    }

    #[tokio::test]
    async fn precompressed_index_variant_escaping_root_is_rejected() {
        for (extension, encoding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            let (temp_dir, root_dir) = web_root("index-precomp-outside");
            fs::write(root_dir.join("index.html"), "source")
                .expect("unexpected error writing index file");
            let outside_marker = temp_dir.path().join("outside-marker");
            fs::write(&outside_marker, "beyond").expect("unexpected error writing outside marker");
            symlink(
                &outside_marker,
                root_dir.join(format!("index.html.{extension}")),
            )
            .expect("unexpected error creating escaping variant symlink");

            match request_with(
                &root_dir,
                RequestOpts {
                    uri_path: "/",
                    encoding,
                    ..Default::default()
                },
            )
            .await
            {
                Ok(result) => panic!(
                    "expected escaping {encoding} index variant to be rejected, got {}",
                    result.resp.status()
                ),
                Err(status) => assert_eq!(status, StatusCode::NOT_FOUND),
            }
        }
    }

    #[tokio::test]
    async fn index_file_in_hidden_directory_is_served_when_hidden_files_are_ignored() {
        let (_temp_dir, root_dir) = web_root("hidden-index");
        fs::write(root_dir.join(".hidden-index.html"), "hidden-index")
            .expect("unexpected error writing hidden index file");

        // The appended index file is server-configured, not client-supplied, so
        // it must not be filtered out by the hidden-files policy.
        let result = request_with(
            &root_dir,
            RequestOpts {
                uri_path: "/",
                ignore_hidden_files: true,
                index_files: &[".hidden-index.html"],
                ..Default::default()
            },
        )
        .await
        .expect("expected configured hidden index file to be served");

        assert_eq!(body_of(result).await, b"hidden-index");
    }

    #[tokio::test]
    async fn hidden_directory_request_is_still_rejected_when_hidden_files_are_ignored() {
        let (_temp_dir, root_dir) = web_root("hidden-dir");
        let hidden_dir = root_dir.join(".hidden");
        fs::create_dir(&hidden_dir).expect("unexpected error creating hidden directory");
        fs::write(hidden_dir.join("index.html"), "hidden-dir-index")
            .expect("unexpected error writing hidden directory index");

        match request_with(
            &root_dir,
            RequestOpts {
                uri_path: "/.hidden/",
                ignore_hidden_files: true,
                ..Default::default()
            },
        )
        .await
        {
            Ok(result) => panic!(
                "expected hidden directory request to be rejected, got {}",
                result.resp.status()
            ),
            Err(status) => assert_eq!(status, StatusCode::NOT_FOUND),
        }
    }

    #[cfg(feature = "experimental")]
    #[tokio::test]
    async fn precompressed_variant_is_not_inserted_into_memory_cache() {
        use crate::mem_cache::cache::{CACHE_STORE, MemCacheOpts};
        use compact_str::CompactString;

        let (_temp_dir, root_dir) = web_root("precomp-mem-cache");
        fs::write(root_dir.join("app.js"), "source").expect("unexpected error writing source file");
        let precompressed_body = b"pre-compressed-bytes";
        fs::write(root_dir.join("app.js.gz"), precompressed_body)
            .expect("unexpected error writing pre-compressed file");

        // `CACHE_STORE` is process-global and may already be initialized by
        // another test; either way we only need a live store to assert against.
        let _ = CACHE_STORE.set(
            mini_moka::sync::Cache::builder()
                .max_capacity(8)
                .time_to_live(std::time::Duration::from_secs(60))
                .build(),
        );
        let store = CACHE_STORE
            .get()
            .expect("expected an in-memory cache store");

        let memory_cache = MemCacheOpts::new(8);
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            "gzip".parse().expect("unexpected invalid encoding"),
        );

        let result = handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &root_dir,
            uri_path: "/app.js",
            uri_query: None,
            memory_cache: Some(&memory_cache),
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &crate::directory_listing::DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: true,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("expected pre-compressed response");

        assert_eq!(body_of(result).await, precompressed_body);

        // The cache is keyed by the *original* path and its lookup runs before
        // content negotiation, so a cached pre-compressed body would be served
        // to clients that never accepted that encoding.
        let key = CompactString::from(
            root_dir
                .join("app.js")
                .to_str()
                .expect("unexpected non-UTF-8 web root"),
        );
        assert!(
            store.get(&key).is_none(),
            "pre-compressed body was inserted into the in-memory cache store"
        );
    }

    #[tokio::test]
    async fn containment_cache_is_bypassed_when_symlinks_are_followed() {
        let (_temp_dir, root_dir) = web_root("containment-cache-bypassed");
        fs::write(root_dir.join("app.js"), "source").expect("unexpected error writing source file");

        // With symlinks followed, every request must re-canonicalize so a
        // symlink retargeted after an earlier "contained" decision is caught.
        reset_canonicalize_calls();
        for _ in 0..3 {
            request(&root_dir, "identity", false)
                .await
                .expect("expected a successful response");
        }
        assert_eq!(
            canonicalize_calls(),
            3,
            "containment decisions were reused while symlinks are followed"
        );
    }

    #[tokio::test]
    async fn containment_cache_is_reused_when_symlinks_are_disabled() {
        let (_temp_dir, root_dir) = web_root("containment-cache-reused");
        fs::write(root_dir.join("app.js"), "source").expect("unexpected error writing source file");

        // With symlinks disabled, `enforce_symlink_policy` re-validates every
        // path component on every request, so reusing a positive containment
        // decision cannot mask a retargeted symlink.
        reset_canonicalize_calls();
        for _ in 0..3 {
            request(&root_dir, "identity", true)
                .await
                .expect("expected a successful response");
        }
        assert_eq!(
            canonicalize_calls(),
            1,
            "containment decisions were not reused while symlinks are disabled"
        );
    }
}
