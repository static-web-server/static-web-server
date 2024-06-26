// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides in-memory files cache functionality with eviction policy
//! using the SIEVE algorithm and TTL (Time-to-live) support.
//!

use bytes::Bytes;
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, LastModified,
};
use hyper::{Body, Response, StatusCode};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use sieve_cache::SieveCache;
use std::io::{Read, Seek, SeekFrom};
use std::time::{Duration, Instant};

use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::fs::stream::FileStreamLite;
use crate::handler::RequestHandlerOpts;
use crate::response::{bytes_range, BadRangeError};
use crate::Result;

/// Global cache that stores all files in memory.
/// It provides eviction policy using the SIEVE algorithm and TTL (Time-to-live) support.
pub(crate) static CACHE_STORE: OnceCell<Mutex<SieveCache<String, MemFile>>> = OnceCell::new();

/// It defines the in-memory files cache options.
pub struct MemCacheOpts {
    /// The maximum size of the cache entries.
    pub max_size: usize,
    /// The maximum size per file in bytes.
    pub file_max_size: u64,
    /// The TTL per file in seconds.
    pub file_ttl: u64,
}

impl MemCacheOpts {
    /// Creates a new instance of `MemCacheOpts`.
    #[inline]
    pub fn new(max_size: usize, file_max_size: u64, file_ttl: u64) -> Self {
        Self {
            max_size,
            file_max_size: 1024 * 1024 * file_max_size,
            file_ttl,
        }
    }
}

/// Make sure to initialize the in-memory cache store.
pub(crate) fn init(opts: Option<MemCacheOpts>, handler_opts: &mut RequestHandlerOpts) -> Result {
    // TODO: better options print -> Result
    server_info!("in-memory files cache: enabled={}", opts.is_some());

    if let Some(o) = opts {
        let cache = match SieveCache::new(o.max_size) {
            Ok(v) => v,
            Err(err) => bail!(err),
        };
        if CACHE_STORE.set(Mutex::new(cache)).is_err() {
            bail!("unable to initialize the in-memory cache store")
        }
        tracing::debug!("the in-memory cache store was initialized successfully");
        handler_opts.memory_cache = Some(o);
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct MemFileTempOpts {
    pub(crate) file_path: String,
    pub(crate) content_type: ContentType,
    pub(crate) last_modified: Option<LastModified>,
    pub(crate) file_ttl: u64,
}

impl MemFileTempOpts {
    pub(crate) fn new(
        file_ttl: u64,
        file_path: String,
        content_type: ContentType,
        last_modified: Option<LastModified>,
    ) -> Self {
        Self {
            file_path,
            content_type,
            last_modified,
            file_ttl,
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
    /// Expiration time (TTL) of the current file in memory.
    expiration: Instant,
}

impl MemFile {
    #[inline]
    pub(crate) fn new(
        data: Bytes,
        buf_size: usize,
        content_type: ContentType,
        last_modified: Option<LastModified>,
        file_ttl: u64,
    ) -> Self {
        let expiration = Instant::now() + Duration::new(file_ttl, 0);
        Self {
            data,
            buf_size,
            content_type,
            last_modified,
            expiration,
        }
    }

    #[inline]
    pub(crate) fn has_expired(&self) -> bool {
        Instant::now() > self.expiration
    }

    pub(crate) fn response_body(&self, headers: &HeaderMap) -> Result<Response<Body>, StatusCode> {
        let conditionals = ConditionalHeaders::new(headers);
        let modified = self.last_modified;

        match conditionals.check(modified) {
            ConditionalBody::NoBody(resp) => Ok(resp),
            ConditionalBody::WithBody(range) => {
                let data = self.data.clone();
                let mut len = data.len() as u64;
                let mut reader = std::io::Cursor::new(data);
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

                        let body = Body::wrap_stream(FileStreamLite { reader, buf_size });
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
