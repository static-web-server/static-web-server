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
use hyper::{Body, Response, StatusCode};
use mini_moka::sync::Cache;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::fs::stream::FileStream;
use crate::handler::RequestHandlerOpts;
use crate::response::{bytes_range, BadRangeError};
use crate::Result;

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

impl MemCacheOpts {
    /// Creates a new instance of `MemCacheOpts`.
    #[inline]
    pub fn new(max_file_size: u64) -> Self {
        Self {
            max_file_size: 1024 * 1024 * max_file_size,
        }
    }
}

/// Make sure to initialize the in-memory cache store.
pub(crate) fn init(handler_opts: &mut RequestHandlerOpts) -> Result {
    if let Some(advanced_opts) = handler_opts.advanced_opts.as_ref() {
        if let Some(opts) = advanced_opts.memory_cache.as_ref() {
            // TODO: define maximum values

            // Default 256 entries
            let capacity = opts.capacity.unwrap_or(256);
            // Default 1h
            let ttl = opts.ttl.unwrap_or(3600);
            // Default 5min
            let tti = opts.tti.unwrap_or(300);
            // Default 8mb
            let max_file_size = opts.max_file_size.unwrap_or(8192);

            server_info!(
                "in-memory cache (experimental): enabled=true, capacity={capacity}, ttl={ttl}, tti={tti}, max_file_size={max_file_size}"
            );

            let mem_opts = MemCacheOpts::new(max_file_size);

            let cache = Cache::builder()
                .max_capacity(capacity)
                // Time to live (TTL): 30 minutes
                .time_to_live(Duration::from_secs(ttl))
                // Time to idle (TTI):  5 minutes
                .time_to_idle(Duration::from_secs(tti))
                .build();

            if CACHE_STORE.set(cache).is_err() {
                bail!("unable to initialize the in-memory cache store")
            }

            handler_opts.memory_cache = Some(mem_opts);

            return Ok(());
        }
    }

    server_info!("in-memory cache (experimental): enabled=false");

    Ok(())
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
) -> Option<Result<Response<Body>, StatusCode>> {
    let file_path_str = file_path.to_str().or(None)?;

    let store = CACHE_STORE.get().unwrap();
    match store.get::<CompactString>(&file_path_str.into()) {
        Some(mem_file) => {
            tracing::debug!(
                "file `{}` found in the in-memory cache store, returning it directly",
                file_path_str
            );
            Some(mem_file.response_body(headers_opt))
        }
        _ => {
            tracing::debug!(
                "file `{}` was not found in the in-memory cache store, continuing",
                file_path_str
            );
            // If a file is not found in the store then continue
            // with the normal flow and wait on first file read
            if let Err(err) = CACHE_PERMIT.acquire().await {
                tracing::error!("error trying to acquire permit on first read: {:?}", err);
                return Some(Err(StatusCode::INTERNAL_SERVER_ERROR));
            }
            None
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

                        let body = Body::wrap_stream(FileStream { reader, buf_size });
                        let mut resp = Response::new(body);

                        if sub_len != len {
                            *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                            resp.headers_mut().typed_insert(
                                match ContentRange::bytes(start..end, len) {
                                    Ok(range) => range,
                                    Err(err) => {
                                        tracing::error!("invalid content range error: {:?}", err);
                                        let mut resp = Response::new(Body::empty());
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
                        let mut resp = Response::new(Body::empty());
                        *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                        resp.headers_mut()
                            .typed_insert(ContentRange::unsatisfied_bytes(len));
                        Ok(resp)
                    })
            }
        }
    }
}
