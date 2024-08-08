// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides in-memory files cache functionality with expiration policies support
//! such as Time to live (TTL) and Time to idle (TTI).
//!
//! Admission to a cache is controlled by the Least Frequently Used (LFU) policy.
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
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::fs::stream::FileStream;
use crate::handler::RequestHandlerOpts;
use crate::response::{bytes_range, BadRangeError};
use crate::Result;

/// Global cache that stores all files in memory.
/// It provides expiration policies like Time to live (TTL) and Time to idle (TTI) support.
pub(crate) static CACHE_STORE: OnceLock<Cache<CompactString, Arc<MemFile>>> = OnceLock::new();

/// It defines the in-memory files cache options.
pub struct MemCacheOpts {
    /// The maximum size of the cache entries.
    pub max_size: u64,
    /// The maximum size per file in bytes.
    pub file_max_size: u64,
    /// The TTL per file in seconds.
    pub file_ttl: u64,
}

impl MemCacheOpts {
    /// Creates a new instance of `MemCacheOpts`.
    #[inline]
    pub fn new(max_size: u64, file_max_size: u64, file_ttl: u64) -> Self {
        Self {
            max_size,
            file_max_size: 1024 * 1024 * file_max_size,
            file_ttl,
        }
    }
}

/// Make sure to initialize the in-memory cache store.
pub(crate) fn init(
    enabled: bool,
    opts: Option<MemCacheOpts>,
    handler_opts: &mut RequestHandlerOpts,
) -> Result {
    if enabled {
        server_info!("in-memory files cache: enabled={enabled}");

        // TODO: provide options via config
        // TODO: better options printing
        if let Some(opts) = opts.as_ref() {
            let cache = Cache::builder()
                .max_capacity(opts.max_size)
                // Time to live (TTL): 30 minutes
                .time_to_live(Duration::from_secs(opts.file_ttl))
                // Time to idle (TTI):  5 minutes
                .time_to_idle(Duration::from_secs(5 * 60))
                .build();

            if CACHE_STORE.set(cache).is_err() {
                bail!("unable to initialize the in-memory cache store")
            }
        }

        handler_opts.memory_cache = opts;
    }
    Ok(())
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
