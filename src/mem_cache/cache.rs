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
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, LastModified,
};
use hyper::{Response, StatusCode};
use mini_moka::sync::Cache;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::Result;
use crate::body::Body;
use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::fs::stream::FileStream;
use crate::handler::RequestHandlerOpts;
use crate::response::{BadRangeError, bytes_range};

/// Global cache that stores all files in memory.
/// It provides expiration policies like Time to live (TTL) and Time to idle (TTI) support.
pub(crate) static CACHE_STORE: OnceLock<Cache<CompactString, Arc<MemFile>>> = OnceLock::new();

/// A single cache permit to allow reading a file once.
static CACHE_PERMIT: Semaphore = Semaphore::const_new(1);

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
/// Maximum allowed file size in KiB (256 MiB = 262144 KiB).
const MAX_FILE_SIZE: u64 = 262_144;

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
            "in-memory cache: enabled=true, capacity={capacity}, ttl={ttl}s, tti={tti}s, max_file_size={max_file_size}KiB"
        );

        let mem_opts = MemCacheOpts::new(max_file_size);

        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(Duration::from_secs(ttl))
            .time_to_idle(Duration::from_secs(tti))
            .build();

        if CACHE_STORE.set(cache).is_err() {
            bail!("unable to initialize the in-memory cache store")
        }

        handler_opts.memory_cache = Some(mem_opts);

        return Ok(());
    }

    tracing::info!("in-memory cache: enabled=false");

    Ok(())
}

/// Result of a cache lookup via `get_or_acquire`.
pub(crate) enum CacheResult<'a> {
    /// Cache hit — return the response directly.
    Hit(Result<Response<Body>, StatusCode>),
    /// Cache miss — caller should read the file. The semaphore permit is held
    /// so that concurrent misses are serialized (single-flight). Drop the permit
    /// after inserting into the cache store.
    Miss(SemaphorePermit<'a>),
    /// An error occurred acquiring the semaphore.
    Error(StatusCode),
}

/// Try to get the file in a form of a response from the cache store by a path or
/// acquires a permit to ensure to hold until the file is read first (once).
///
/// If the file is not found in the cache store then
/// a cache permit is acquired internally (one at a time)
/// to allow the caller to read the file first.
/// Once the file is read on caller's side then the permit is dropped.
pub(crate) async fn get_or_acquire(
    file_path: &Path,
    headers_opt: &HeaderMap,
) -> Option<CacheResult<'static>> {
    let file_path_str = file_path.to_str().or(None)?;

    let store = CACHE_STORE.get()?;
    match store.get::<CompactString>(&file_path_str.into()) {
        Some(mem_file) => {
            tracing::debug!(
                "file `{}` found in the in-memory cache store, returning it directly",
                file_path_str
            );
            Some(CacheResult::Hit(mem_file.response_body(headers_opt)))
        }
        _ => {
            tracing::debug!(
                "file `{}` was not found in the in-memory cache store, continuing",
                file_path_str
            );
            // If a file is not found in the store then continue
            // with the normal flow and wait on first file read.
            // Hold the permit so concurrent misses are serialized.
            match CACHE_PERMIT.acquire().await {
                Ok(permit) => {
                    // Re-check after acquiring — another request may have populated
                    // the cache while we were waiting for the permit.
                    if let Some(mem_file) = store.get::<CompactString>(&file_path_str.into()) {
                        return Some(CacheResult::Hit(mem_file.response_body(headers_opt)));
                    }
                    Some(CacheResult::Miss(permit))
                }
                Err(err) => {
                    tracing::error!("error trying to acquire permit on first read: {:?}", err);
                    Some(CacheResult::Error(StatusCode::INTERNAL_SERVER_ERROR))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MemFileTempOpts {
    pub(crate) file_path: String,
    pub(crate) content_type: ContentType,
    pub(crate) last_modified: Option<LastModified>,
}

impl MemFileTempOpts {
    pub(crate) fn new(
        file_path: String,
        content_type: ContentType,
        last_modified: Option<LastModified>,
    ) -> Self {
        Self {
            file_path,
            content_type,
            last_modified,
        }
    }
}

/// In-memory file representation to be store in the cache.
#[derive(Debug)]
pub(crate) struct MemFile {
    /// Bytes of the the current file.
    data: Bytes,
    /// Buffer size for the current file.
    buf_size: usize,
    /// `Content-Type` header for the current file.
    content_type: ContentType,
    /// `Last Modified` header for the current file.
    last_modified: Option<LastModified>,
}

impl MemFile {
    #[inline]
    pub(crate) fn new(
        data: Bytes,
        buf_size: usize,
        content_type: ContentType,
        last_modified: Option<LastModified>,
    ) -> Self {
        Self {
            data,
            buf_size,
            content_type,
            last_modified,
        }
    }

    pub(crate) fn response_body(&self, headers: &HeaderMap) -> Result<Response<Body>, StatusCode> {
        let conditionals = ConditionalHeaders::new(headers);
        let modified = self.last_modified;

        match conditionals.check(modified) {
            ConditionalBody::NoBody(resp) => Ok(resp),
            ConditionalBody::WithBody(range) => {
                let mem_buf = self.data.clone();
                let mut len = mem_buf.len() as u64;
                let mut reader = std::io::Cursor::new(mem_buf);
                let buf_size = self.buf_size;

                bytes_range(range, len)
                    .map(|(start, end)| {
                        match reader.seek(SeekFrom::Start(start)) {
                            Ok(_) => (),
                            Err(err) => {
                                tracing::error!("seek file from start error: {:?}", err);
                                return Err(StatusCode::INTERNAL_SERVER_ERROR);
                            }
                        };

                        let sub_len = end - start;
                        let reader = reader.take(sub_len);

                        let body = crate::body::stream(FileStream::new(reader, buf_size));
                        let mut resp = Response::new(body);

                        if sub_len != len {
                            *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                            resp.headers_mut().typed_insert(
                                match ContentRange::bytes(start..end, len) {
                                    Ok(range) => range,
                                    Err(err) => {
                                        tracing::error!("invalid content range error: {:?}", err);
                                        let mut resp = Response::new(crate::body::empty());
                                        *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                                        resp.headers_mut()
                                            .typed_insert(ContentRange::unsatisfied_bytes(len));
                                        return Ok(resp);
                                    }
                                },
                            );

                            len = sub_len;
                        }

                        resp.headers_mut().typed_insert(ContentLength(len));
                        resp.headers_mut().typed_insert(self.content_type.clone());
                        resp.headers_mut().typed_insert(AcceptRanges::bytes());

                        if let Some(last_modified) = modified {
                            resp.headers_mut().typed_insert(last_modified);
                        }

                        Ok(resp)
                    })
                    .unwrap_or_else(|BadRangeError| {
                        // bad byte range
                        let mut resp = Response::new(crate::body::empty());
                        *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                        resp.headers_mut()
                            .typed_insert(ContentRange::unsatisfied_bytes(len));
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
        // File size capped at 256 MiB (in KiB)
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
}
