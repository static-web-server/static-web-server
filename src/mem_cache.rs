// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides in-memory files cache functionality.
//!

use bytes::BytesMut;
use compact_str::CompactString;
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, LastModified,
};
use hyper::{Body, Response, StatusCode};
use once_cell::sync::OnceCell;
use sieve_cache::SieveCache;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::file_response::{bytes_range, BadRangeError};
use crate::file_stream::FileStream;
use crate::Result;

/// Global cache that stores all files in memory.
/// It provides eviction policy using the SIEVE algorithm and TTL (Time-to-live) support.
pub(crate) static CACHE_STORE: OnceCell<Mutex<SieveCache<CompactString, MemFile>>> =
    OnceCell::new();

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
    /// Creates a new isntance of `MemCacheOpts`.
    pub fn new(max_size: usize, file_max_size: u64, file_ttl: u64) -> Self {
        Self {
            max_size,
            file_max_size: 1024 * 1024 * file_max_size,
            file_ttl,
        }
    }
}

/// Make sure to initialize the in-memory cache store.
pub fn init_store(opts: &MemCacheOpts) -> Result {
    let cache = match SieveCache::new(opts.max_size) {
        Ok(v) => v,
        Err(err) => bail!(err),
    };
    if CACHE_STORE.set(Mutex::new(cache)).is_err() {
        bail!("unable to initialize the in-memory cache store")
    }
    tracing::debug!("the in-memory cache store was initialized successfully");

    Ok(())
}

/// In-memory file representation to be store in the cache.
#[derive(Debug)]
pub(crate) struct MemFile {
    /// Bytes of the the current file.
    pub data: BytesMut,
    /// Buffer size for the current file.
    pub buf_size: usize,
    /// `Content-Type` header for the current file.
    pub content_type: ContentType,
    /// `Last Modified` header for the current file.
    pub last_modified: Option<LastModified>,
    /// Expiration time (TTL) of the current file in memory.
    pub expiration: Instant,
}

impl MemFile {
    pub(crate) fn new(
        len: u64,
        buf_size: usize,
        content_type: ContentType,
        last_modified: Option<LastModified>,
        file_ttl: u64,
    ) -> Self {
        Self {
            data: BytesMut::with_capacity(len as usize),
            buf_size,
            content_type,
            last_modified,
            expiration: Instant::now() + Duration::new(file_ttl, 0),
        }
    }

    pub(crate) fn has_expired(&self) -> bool {
        Instant::now() > self.expiration
    }

    pub(crate) fn response_body(&self, headers: &HeaderMap) -> Result<Response<Body>, StatusCode> {
        let conditionals = ConditionalHeaders::new(headers);
        let modified = self.last_modified;

        match conditionals.check(modified) {
            ConditionalBody::NoBody(resp) => Ok(resp),
            ConditionalBody::WithBody(range) => {
                let buf = self.data.clone().freeze();
                let mut len = buf.len() as u64;
                let mut reader = std::io::Cursor::new(buf);
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
                        let stream = FileStream {
                            reader,
                            buf_size,
                            file_path: None,
                        };
                        let body = Body::wrap_stream(stream);
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
