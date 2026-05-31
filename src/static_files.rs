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
use std::io;
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
    /// proven the probe is contained within `base_path`. The cache is
    /// not invalidated on filesystem changes. This is acceptable for
    /// a static-file server: the worst case is a stale "OK" decision
    /// after an admin renames a directory to a symlink, which is a
    /// transient state requiring filesystem changes outside SWS.
    static CONTAINMENT_CACHE: RefCell<HashSet<PathBuf>> =
        RefCell::new(HashSet::with_capacity(64));
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
        metadata,
        is_dir,
        precompressed_variant,
        file: pre_opened,
    } = get_composed_file_metadata(
        &mut file_path,
        headers_opt,
        opts.compression_static,
        opts.index_files,
    )?;

    let mut file_path_temp = file_path.clone();
    if is_dir {
        file_path_temp.pop();
    }

    let file_path_relative = file_path_temp.strip_prefix(opts.base_path).map_err(|err| {
        tracing::error!(
            "unable to strip prefix from file path '{}': {}",
            file_path.display(),
            err,
        );
        StatusCode::NOT_FOUND
    })?;

    let file_path_resolved =
        match CONTAINMENT_CACHE.with(|c| c.borrow().contains(file_path_temp.as_path())) {
            true => file_path_temp.clone(),
            false => {
                let resolved = file_path_temp.canonicalize().map_err(|err| {
                    tracing::error!(
                        "unable to resolve '{}' symlink path: {}",
                        file_path_temp.display(),
                        err,
                    );
                    StatusCode::NOT_FOUND
                })?;

                // a. Fast path: when `base_path` is already canonical (the
                // production case), the resolved file path will share its
                // prefix and we avoid a per-request `canonicalize` syscall on
                // the base directory.
                if resolved.starts_with(opts.base_path) {
                    cache_safe_probe(file_path_temp.as_path());
                    resolved
                } else {
                    // b. Fallback: canonicalize the base and retry the check.
                    let base_path = opts.base_path.canonicalize().map_err(|err| {
                        tracing::error!(
                            "unable to resolve '{}' base path: {}",
                            opts.base_path.display(),
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
                    cache_safe_probe(file_path_temp.as_path());
                    resolved
                }
            }
        };
    // Silence unused warning when fast path is hit on subsequent requests.
    let _ = &file_path_resolved;

    if opts.disable_symlinks {
        // Check if the whole path or any path component contains a symlink.
        // Note that this could be expensive as it requires filesystem access for each path component.
        let has_symlink = file_path_relative
            .contains_symlink(opts.base_path)
            .map_err(|err| {
                tracing::error!(
                    "unable to check if file path '{}' contains symlink: {}",
                    file_path_relative.display(),
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
    }

    // Check for a hidden file/directory (dotfile) and ignore it if feature enabled
    if opts.ignore_hidden_files && file_path_relative.is_hidden() {
        tracing::trace!(
            "considering hidden file {} as not found",
            file_path.display()
        );
        return Err(StatusCode::NOT_FOUND);
    }

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
    if is_dir && opts.dir_listing && !file_path.exists() {
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
    if let Some(precompressed_meta) = precompressed_variant {
        let (precomp_path, precomp_encoding) = precompressed_meta;
        // Pre-opened handle (if any) refers to the original file we are
        // about to replace with the precompressed variant; just drop it.
        drop(pre_opened);
        let mut resp = file_reply(
            headers_opt,
            file_path,
            &metadata,
            Some(precomp_path),
            None,
            #[cfg(feature = "experimental")]
            opts.memory_cache,
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

    #[cfg(feature = "experimental")]
    let resp = file_reply(
        headers_opt,
        file_path,
        &metadata,
        None,
        pre_opened,
        opts.memory_cache,
    )?;

    #[cfg(not(feature = "experimental"))]
    let resp = file_reply(headers_opt, file_path, &metadata, None, pre_opened)?;

    Ok(StaticFileResponse {
        resp,
        file_path: resp_file_path,
    })
}

/// Returns the final composed metadata containing
/// the current `file_path` with its file metadata
/// as well as its optional pre-compressed variant.
fn get_composed_file_metadata<'a>(
    mut file_path: &'a mut PathBuf,
    headers: &'a HeaderMap<HeaderValue>,
    compression_static: bool,
    mut index_files: &'a [&'a str],
) -> Result<FileMetadata<'a>, StatusCode> {
    tracing::trace!("getting metadata for file {}", file_path.display());

    // Try to find the file path on the file system
    match try_metadata(file_path) {
        Ok((mut metadata, is_dir)) => {
            // The optional pre-opened file for `file_path`. When `Some`, the
            // response pipeline reuses this handle instead of issuing an
            // extra `open(2)` syscall. We only populate it when the index
            // file is resolved via `try_file_open` below.
            let mut opened_file = None;
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

                    // Try to open the appended index file directly.
                    // `try_file_open` performs a single `open(2)` + `fstat(2)`
                    // instead of `stat(2)` followed by `open(2)` later in
                    // `file_reply`, saving one path-resolving syscall on the
                    // hot path.
                    if let Ok((file, meta)) = try_file_open(file_path) {
                        metadata = meta;
                        opened_file = Some(file);
                        resolved_exists = true;
                        break;
                    }

                    // We remove only the appended index file
                    file_path.pop();
                    let new_meta: Option<Metadata>;
                    (file_path, new_meta) = try_metadata_with_html_suffix(file_path);
                    if let Some(new_meta) = new_meta {
                        metadata = new_meta;
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

            // If we are going to serve a precompressed variant, the
            // pre-opened file points to the *original* file which won't be
            // streamed; drop it so `file_reply` opens the precomp file.
            if precompressed_variant.is_some() {
                opened_file = None;
            }

            Ok(FileMetadata {
                file_path,
                metadata,
                is_dir,
                precompressed_variant,
                file: opened_file,
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

            let Some(new_meta) = new_meta else {
                // Neither the original path nor its `.html` sibling exists.
                // Return the original error without probing for compressed
                // variants of non-existent files.
                return Err(err);
            };

            // The `.html` sibling exists. Only now is it worth probing for
            // its pre-compressed sibling (`/article.html.br`, etc.).
            let precompressed_variant = compression_static
                .then(|| compression_static::precompressed_variant(file_path, headers))
                .flatten()
                .map(|p| (p.file_path, p.encoding));

            Ok(FileMetadata {
                file_path,
                metadata: new_meta,
                is_dir: false,
                precompressed_variant,
                file: None,
            })
        }
    }
}

/// Reply with the corresponding file content taking into account
/// its precompressed variant if any.
/// The `path` param should contains always the original requested file path and
/// the `meta` param value should corresponds to it.
/// However, if `path_precompressed` contains some value then
/// the `meta` param  value will belong to the `path_precompressed` (precompressed file variant).
fn file_reply<'a>(
    headers: &'a HeaderMap<HeaderValue>,
    path: &'a PathBuf,
    meta: &'a Metadata,
    path_precompressed: Option<PathBuf>,
    pre_opened: Option<File>,
    #[cfg(feature = "experimental")] memory_cache: Option<&'a MemCacheOpts>,
) -> Result<Response<Body>, StatusCode> {
    let conditionals = ConditionalHeaders::new(headers);

    // Reuse the pre-opened handle when serving the original file. For
    // precompressed variants the open target differs, so we open the
    // precomp file ourselves (and the caller dropped `pre_opened`).
    let file_result = match (path_precompressed.as_deref(), pre_opened) {
        (None, Some(file)) => Ok(file),
        (Some(precomp_path), _) => File::open(precomp_path),
        (None, None) => File::open(path),
    };

    match file_result {
        Ok(file) => {
            #[cfg(feature = "experimental")]
            let resp = response_body(file, path, meta, conditionals, memory_cache);

            #[cfg(not(feature = "experimental"))]
            let resp = response_body(file, path, meta, conditionals);

            resp
        }
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!("file can't be opened or not found: {:?}", path.display());
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!("file permission denied: {:?}", path.display());
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!("file open error (path={:?}): {} ", path.display(), err);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Err(status)
        }
    }
}
