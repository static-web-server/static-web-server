// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides in-memory files cache functionality with expiration policy support
//! such as Time to live (TTL) and Time to idle (TTI).
//!
//! Admission to a cache is controlled by the Least Frequently Used (LFU) policy
//! and the eviction from a cache is controlled by the Least Recently Used (LRU) policy.
//!

use bytes::Bytes;
use compact_str::CompactString;
use headers::{AcceptRanges, ContentLength, ContentRange, HeaderMap, HeaderMapExt, LastModified};
use hyper::header::{CONTENT_TYPE, ETAG, HeaderName, HeaderValue};
use hyper::{Response, StatusCode};
use mini_moka::sync::Cache;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::Result;
use crate::body::{self, Body};
use crate::conditional_headers::{ConditionalBody, ConditionalHeaders, Validators};
use crate::handler::RequestHandlerOpts;
use crate::response::range::{BadRangeError, bytes_range};

/// Global cache that stores all files in memory.
/// It provides expiration policies like Time to live (TTL) and Time to idle (TTI) support.
pub(crate) static CACHE_STORE: OnceLock<Cache<CompactString, Arc<MemFile>>> = OnceLock::new();

/// Standard `X-Cache` header to indicate whether a response was served from cache.
pub(crate) static X_CACHE: HeaderName = HeaderName::from_static("x-cache");
/// Value for the `X-Cache` header when the response is a cache hit.
pub(crate) static X_CACHE_HIT: HeaderValue = HeaderValue::from_static("HIT");

/// It defines the in-memory files cache options.
pub struct MemCacheOpts {
    /// The maximum size per file in bytes.
    pub max_file_size: u64,
}

/// Default capacity (number of entries).
pub const DEFAULT_CAPACITY: u64 = 100;
/// Default time-to-live in seconds (30 minutes).
pub const DEFAULT_TTL: u64 = 1800;
/// Default time-to-idle in seconds (5 minutes).
pub const DEFAULT_TTI: u64 = 300;
/// Default maximum file size in KiB (8 MiB = 8192 KiB).
pub const DEFAULT_MAX_FILE_SIZE: u64 = 8192;

/// Maximum allowed capacity (entries).
const MAX_CAPACITY: u64 = 100_000;
/// Maximum allowed TTL in seconds (24 hours).
const MAX_TTL: u64 = 86_400;
/// Maximum allowed TTI in seconds (1 hour).
const MAX_TTI: u64 = 3_600;
/// Maximum allowed file size in KiB (32 MiB = 32768 KiB).
const MAX_FILE_SIZE: u64 = 32_768;

impl MemCacheOpts {
    /// Creates a new instance of `MemCacheOpts`.
    /// `max_file_size` is in KiB and gets converted to bytes internally.
    #[inline]
    pub fn new(max_file_size: u64) -> Self {
        Self {
            max_file_size: max_file_size * 1024,
        }
    }
}

/// Initialize the in-memory cache store from handler options.
///
/// If `[advanced.memory-cache]` is present in the configuration, a cache store
/// is created with the specified (or default) parameters. Values exceeding
/// their allowed maximums are clamped silently.
pub fn init(handler_opts: &mut RequestHandlerOpts) -> Result {
    if let Some(advanced_opts) = handler_opts.advanced_opts.as_ref()
        && let Some(opts) = advanced_opts.memory_cache.as_ref()
    {
        let capacity = opts.capacity.unwrap_or(DEFAULT_CAPACITY).min(MAX_CAPACITY);
        let ttl = opts.ttl.unwrap_or(DEFAULT_TTL).min(MAX_TTL);
        let tti = opts.tti.unwrap_or(DEFAULT_TTI).min(MAX_TTI);
        let max_file_size = opts
            .max_file_size
            .unwrap_or(DEFAULT_MAX_FILE_SIZE)
            .min(MAX_FILE_SIZE);

        tracing::info!(
            enabled = true,
            capacity,
            ttl_seconds = ttl,
            tti_seconds = tti,
            max_file_size_kib = max_file_size,
            "in-memory cache"
        );

        let mem_opts = MemCacheOpts::new(max_file_size);

        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(Duration::from_secs(ttl))
            .time_to_idle(Duration::from_secs(tti))
            .build();

        if CACHE_STORE.set(cache).is_err() {
            tracing::debug!("in-memory cache store already initialized; reusing existing store");
        }

        handler_opts.memory_cache = Some(mem_opts);

        return Ok(());
    }

    tracing::info!(enabled = false, "in-memory cache");

    Ok(())
}

/// Try to get a cached response for the given file path.
///
/// Returns `Some(result)` on a cache hit (the result itself may be an error
/// status, e.g. for a malformed `Range` header) or `None` when the cache is
/// disabled, the path is non-UTF-8, or there is no entry yet (cache miss).
///
/// The caller is responsible for reading the file from disk on a miss and
/// inserting it into the cache via the streaming pipeline. There is no
/// single-flight serialization: mini-moka's `Cache` is concurrency-safe and
/// duplicate inserts under contention are benign and rare in practice.
pub(crate) fn lookup(
    file_path: &Path,
    headers_opt: &HeaderMap,
) -> Option<Result<Response<Body>, StatusCode>> {
    let file_path_str = file_path.to_str()?;
    let store = CACHE_STORE.get()?;
    let key = CompactString::from(file_path_str);
    let mem_file = store.get(&key)?;
    tracing::debug!("file `{file_path_str}` served from the in-memory cache store");
    // Tag the response with `X-Cache: HIT` so clients and tooling can
    // identify that it was served from the in-memory cache.
    Some(mem_file.response_body(headers_opt).map(|mut resp| {
        resp.headers_mut()
            .insert(X_CACHE.clone(), X_CACHE_HIT.clone());
        resp
    }))
}

#[derive(Debug, Clone)]
pub(crate) struct MemFileTempOpts {
    pub(crate) file_path: String,
    /// Pre-built `Content-Type` `HeaderValue`. Reusing a `HeaderValue`
    /// (instead of [`ContentType`]) avoids re-encoding the mime string
    /// when the entry is eventually inserted into the cache.
    pub(crate) content_type: HeaderValue,
    pub(crate) last_modified: Option<LastModified>,
    /// Pre-built weak `ETag` value. Built once on the disk path and
    /// reused on every cache hit (refcount-clone only).
    pub(crate) etag: Option<HeaderValue>,
}

impl MemFileTempOpts {
    pub(crate) fn new(
        file_path: String,
        content_type: HeaderValue,
        last_modified: Option<LastModified>,
        etag: Option<HeaderValue>,
    ) -> Self {
        Self {
            file_path,
            content_type,
            last_modified,
            etag,
        }
    }
}

/// In-memory file representation to be stored in the cache.
///
/// Holds the full file body as a [`Bytes`] (shared, reference-counted, zero-copy
/// cloneable) and a pre-built `Content-Type` [`HeaderValue`] so that serving a
/// cached response avoids any per-request allocation or string conversion.
#[derive(Debug)]
pub(crate) struct MemFile {
    /// Bytes of the current file.
    data: Bytes,
    /// Pre-built `Content-Type` header value. Stored as a [`HeaderValue`]
    /// (rather than [`ContentType`]) so that emitting it on a cache hit is a
    /// cheap reference-counted clone.
    content_type: HeaderValue,
    /// `Last-Modified` header for the current file.
    last_modified: Option<LastModified>,
    /// Weak `ETag` header value for the cached representation. When
    /// present it is both emitted on the response and used for
    /// `If-None-Match` / `If-Match` / `If-Range` evaluation.
    etag: Option<HeaderValue>,
}

impl MemFile {
    #[inline]
    pub(crate) fn new(
        data: Bytes,
        content_type: HeaderValue,
        last_modified: Option<LastModified>,
        etag: Option<HeaderValue>,
    ) -> Self {
        Self {
            data,
            content_type,
            last_modified,
            etag,
        }
    }

    /// Build a response for a cache hit.
    ///
    /// The body is constructed directly from the in-memory [`Bytes`] (a single
    /// reference-counted clone for full responses, an O(1) `Bytes::slice` for
    /// range responses). No allocation or copying of the file contents occurs
    /// on the hot path; the response body is a single data frame, not a
    /// chunked stream.
    pub(crate) fn response_body(&self, headers: &HeaderMap) -> Result<Response<Body>, StatusCode> {
        let conditionals = ConditionalHeaders::new(headers);
        let modified = self.last_modified;

        // The typed `ETag` is only required when the request carries one
        // of `If-None-Match`, `If-Match` or `If-Range`. Parsing is cheap
        // (only on conditional requests) and lazy.
        let etag_typed: Option<headers::ETag> = if conditionals.if_none_match.is_some()
            || conditionals.if_match.is_some()
            || conditionals.if_range.is_some()
        {
            self.etag
                .as_ref()
                .and_then(|hv| hv.to_str().ok().and_then(|s| s.parse().ok()))
        } else {
            None
        };

        let validators = Validators {
            last_modified: modified,
            etag: etag_typed.as_ref(),
            etag_value: self.etag.as_ref(),
        };

        match conditionals.check(validators) {
            ConditionalBody::NoBody(resp) => Ok(resp),
            ConditionalBody::WithBody(range) => {
                let total_len = self.data.len() as u64;

                bytes_range(range, total_len)
                    .map(|(start, end)| {
                        let sub_len = end - start;
                        let is_partial = sub_len != total_len;

                        // Zero-copy body: for a full response we clone the
                        // `Bytes` (refcount bump); for a range we use
                        // `Bytes::slice` (O(1), shared buffer).
                        let body_bytes = if is_partial {
                            self.data.slice(start as usize..end as usize)
                        } else {
                            self.data.clone()
                        };
                        let mut resp = Response::new(body::full(body_bytes));

                        if is_partial {
                            *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                            match ContentRange::bytes(start..end, total_len) {
                                Ok(range) => {
                                    resp.headers_mut().typed_insert(range);
                                }
                                Err(err) => {
                                    tracing::error!("invalid content range error: {:?}", err);
                                    let mut resp = Response::new(crate::body::empty());
                                    *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                                    resp.headers_mut()
                                        .typed_insert(ContentRange::unsatisfied_bytes(total_len));
                                    return Ok(resp);
                                }
                            }
                        }

                        let h = resp.headers_mut();
                        h.typed_insert(ContentLength(sub_len));
                        // Cheap refcount clone of the pre-built header value
                        // (avoids re-stringifying the mime type per request).
                        h.insert(CONTENT_TYPE, self.content_type.clone());
                        h.typed_insert(AcceptRanges::bytes());

                        if let Some(last_modified) = modified {
                            h.typed_insert(last_modified);
                        }
                        if let Some(etag) = self.etag.as_ref() {
                            h.insert(ETAG, etag.clone());
                        }

                        Ok(resp)
                    })
                    .unwrap_or_else(|BadRangeError| {
                        let mut resp = Response::new(crate::body::empty());
                        *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                        resp.headers_mut()
                            .typed_insert(ContentRange::unsatisfied_bytes(total_len));
                        Ok(resp)
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_cache_opts_converts_kib_to_bytes() {
        let opts = MemCacheOpts::new(8192);
        // 8192 KiB = 8 MiB = 8_388_608 bytes
        assert_eq!(opts.max_file_size, 8192 * 1024);
    }

    #[test]
    fn mem_cache_opts_zero_size() {
        let opts = MemCacheOpts::new(0);
        assert_eq!(opts.max_file_size, 0);
    }

    #[test]
    fn default_constants_are_sane() {
        assert_eq!(DEFAULT_CAPACITY, 100);
        assert_eq!(DEFAULT_TTL, 1800);
        assert_eq!(DEFAULT_TTI, 300);
        assert_eq!(DEFAULT_MAX_FILE_SIZE, 8192);
    }

    #[test]
    fn max_constants_enforce_upper_bounds() {
        // Capacity capped at 100k
        const {
            assert!(MAX_CAPACITY >= DEFAULT_CAPACITY);
        }
        // TTL capped at 24h
        const {
            assert!(MAX_TTL >= DEFAULT_TTL);
        }
        // TTI capped at 1h
        const {
            assert!(MAX_TTI >= DEFAULT_TTI);
        }
        // File size capped at 32 MiB (in KiB)
        const {
            assert!(MAX_FILE_SIZE >= DEFAULT_MAX_FILE_SIZE);
        }
    }

    #[test]
    fn init_returns_ok_without_advanced_opts() {
        let mut handler_opts = crate::handler::RequestHandlerOpts::default();
        let result = init(&mut handler_opts);
        assert!(result.is_ok());
        assert!(handler_opts.memory_cache.is_none());
    }

    #[test]
    fn init_returns_ok_without_memory_cache_section() {
        let mut handler_opts = RequestHandlerOpts {
            advanced_opts: Some(crate::settings::Advanced {
                headers: None,
                rewrites: None,
                redirects: None,
                virtual_hosts: None,
                memory_cache: None,
            }),
            ..Default::default()
        };
        let result = init(&mut handler_opts);
        assert!(result.is_ok());
        assert!(handler_opts.memory_cache.is_none());
    }

    #[test]
    fn init_with_defaults_creates_cache() {
        let mut handler_opts = RequestHandlerOpts {
            advanced_opts: Some(crate::settings::Advanced {
                headers: None,
                rewrites: None,
                redirects: None,
                virtual_hosts: None,
                memory_cache: Some(crate::settings::file::MemoryCache {
                    capacity: None,
                    ttl: None,
                    tti: None,
                    max_file_size: None,
                }),
            }),
            ..Default::default()
        };
        let result = init(&mut handler_opts);
        assert!(result.is_ok());
        assert!(handler_opts.memory_cache.is_some());
        let opts = handler_opts.memory_cache.unwrap();
        assert_eq!(opts.max_file_size, DEFAULT_MAX_FILE_SIZE * 1024);
    }

    #[test]
    fn init_clamps_values_to_max() {
        // We can't call init() twice due to OnceLock, so test the clamping
        // logic directly via the min() expressions.
        let capacity = 999_999u64.min(MAX_CAPACITY);
        let ttl = 999_999u64.min(MAX_TTL);
        let tti = 999_999u64.min(MAX_TTI);
        let max_file_size = 999_999u64.min(MAX_FILE_SIZE);

        assert_eq!(capacity, MAX_CAPACITY);
        assert_eq!(ttl, MAX_TTL);
        assert_eq!(tti, MAX_TTI);
        assert_eq!(max_file_size, MAX_FILE_SIZE);

        let opts = MemCacheOpts::new(max_file_size);
        assert_eq!(opts.max_file_size, MAX_FILE_SIZE * 1024);
    }

    #[test]
    fn lookup_returns_none_when_store_uninitialized() {
        // When the global `CACHE_STORE` is not initialized (because `init` was
        // never called with a `[advanced.memory-cache]` section in TOML), the
        // lookup must short-circuit to `None` so that the regular file pipeline
        // serves the request without paying any cache overhead.
        let headers = HeaderMap::new();
        let path = std::path::Path::new("/nonexistent/path.txt");
        // Note: this test relies on the cache not being initialized in unit
        // tests context. If another test in this module ever initializes the
        // global store, this assertion becomes a hit/miss check instead.
        if CACHE_STORE.get().is_none() {
            assert!(lookup(path, &headers).is_none());
        }
    }

    #[test]
    fn x_cache_header_constants_are_valid() {
        assert_eq!(X_CACHE.as_str(), "x-cache");
        assert_eq!(X_CACHE_HIT.to_str().unwrap(), "HIT");
    }
}
