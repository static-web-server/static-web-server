// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to transition files into HTTP responses.
//!

use headers::{ContentRange, HeaderMapExt, LastModified};
use hyper::header::{ACCEPT_RANGES, CONTENT_TYPE, ETAG, HeaderName, HeaderValue};
use hyper::{Response, StatusCode};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::body::Body;
use crate::conditional_headers::{ConditionalBody, ConditionalHeaders, Validators};
use crate::fs::stream::{FileStream, optimal_buf_size};

pub(crate) use range::{BadRangeError, bytes_range};
pub(crate) mod range;

/// Pre-computed `HeaderValue` for `Accept-Ranges: bytes`.
///
/// Reused across all responses, avoids the per-request encode work that
/// `typed_insert(AcceptRanges::bytes())` would otherwise pay.
static ACCEPT_RANGES_BYTES: HeaderValue = HeaderValue::from_static("bytes");

/// Fallback content type used when `mime_guess` returns nothing.
static OCTET_STREAM: HeaderValue = HeaderValue::from_static("application/octet-stream");

thread_local! {
    /// Per-worker-thread cache mapping a file extension to its
    /// `Content-Type` `HeaderValue`.
    ///
    /// Profiling showed `mime_guess::from_path` (an extension lookup using
    /// case-insensitive comparison) plus the per-response
    /// `HeaderValue` construction were a noticeable hot spot on the
    /// no-cache path. Static-file workloads reuse a small set of
    /// extensions, so a thread-local hash map gives a high hit rate
    /// without any locking. `HeaderValue::clone` is cheap (it is either
    /// inline or `Arc`-backed internally).
    static CONTENT_TYPE_CACHE: RefCell<HashMap<Box<str>, HeaderValue>> =
        RefCell::new(HashMap::with_capacity(32));
}

/// Returns the `Content-Type` `HeaderValue` for the given file path,
/// using a per-thread cache keyed by the file extension.
fn content_type_for(path: &Path) -> HeaderValue {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    CONTENT_TYPE_CACHE.with(|cache| {
        if let Some(hv) = cache.borrow().get(ext) {
            return hv.clone();
        }
        let mime = mime_guess::from_ext(ext).first_or_octet_stream();
        let hv = HeaderValue::from_str(mime.essence_str()).unwrap_or_else(|_| OCTET_STREAM.clone());
        cache
            .borrow_mut()
            .insert(ext.to_owned().into_boxed_str(), hv.clone());
        hv
    })
}

/// Inserts a `HeaderName: HeaderValue` pair without the round-trip through
/// `HeaderMapExt::typed_insert`. The `typed_insert` helper internally builds
/// a transient `Vec<HeaderValue>` via the `Header::encode` trait method;
/// for headers we already have as a `HeaderValue` this is wasted work.
#[inline]
fn insert_raw(headers: &mut hyper::HeaderMap, name: HeaderName, value: HeaderValue) {
    headers.insert(name, value);
}

#[cfg(feature = "mem-cache")]
use crate::mem_cache::{
    cache::{MemCacheOpts, MemFileTempOpts},
    stream::MemCacheFileStream,
};

/// It converts a file object into a corresponding HTTP response or
/// returns an error holding an HTTP status code otherwise.
pub(crate) fn response_body(
    mut file: File,
    path: &Path,
    meta: &Metadata,
    conditionals: ConditionalHeaders,
    etag_enabled: bool,
    #[cfg(feature = "mem-cache")] memory_cache: Option<&MemCacheOpts>,
) -> Result<Response<Body>, StatusCode> {
    let mut len = meta.len();
    // If the file's modified time is the UNIX epoch, then it's likely not valid and should
    // not be included in the Last-Modified header to avoid cache revalidation issues.
    let modified = meta
        .modified()
        .ok()
        .filter(|&t| t != std::time::UNIX_EPOCH)
        .map(LastModified::from);

    // Build a weak ETag from `(mtime, size)` once per response. The same
    // header value is used for the body response, the cache entry (when
    // applicable) and the 304 / If-Range short-circuit paths.
    let etag = if etag_enabled {
        crate::etag::build_from_meta(meta)
    } else {
        None
    };
    let (etag_typed, etag_value) = match etag.as_ref() {
        Some((t, v)) => (Some(t), Some(v)),
        None => (None, None),
    };

    let validators = Validators {
        last_modified: modified,
        etag: etag_typed,
        etag_value,
    };

    match conditionals.check(validators) {
        ConditionalBody::NoBody(resp) => Ok(resp),
        ConditionalBody::WithBody(range) => {
            let buf_size = optimal_buf_size(meta);

            bytes_range(range, len)
                .map(|(start, end)| {
                    match file.seek(SeekFrom::Start(start)) {
                        Ok(_) => (),
                        Err(err) => {
                            tracing::error!("seek file from start error: {:?}", err);
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    };

                    let sub_len = end - start;
                    let reader = BufReader::new(file).take(sub_len);

                    let content_type = content_type_for(path);

                    // Select the response body. The in-memory cache pipeline
                    // (`MemCacheFileStream`) is only used for **full-file**
                    // responses (no `Range` header) that fit within the
                    // configured size limit and when the path is valid UTF-8.
                    // Range requests and all other cases use the plain
                    // `FileStream` to avoid wasted allocations and to keep
                    // partial content out of the cache.
                    #[cfg(feature = "mem-cache")]
                    let body = {
                        let is_full_response = sub_len == len;
                        let mem_opts = match (is_full_response, memory_cache, path.to_str()) {
                            (true, Some(opts), Some(path_str)) if len <= opts.max_file_size => {
                                Some(MemFileTempOpts::new(
                                    path_str.to_owned(),
                                    content_type.clone(),
                                    modified,
                                    etag_value.cloned(),
                                ))
                            }
                            _ => None,
                        };
                        match mem_opts {
                            Some(mem_opts) => {
                                tracing::debug!(
                                    "preparing `{}` to be inserted in-memory cache store",
                                    mem_opts.file_path.as_str(),
                                );
                                crate::body::stream(MemCacheFileStream::new(
                                    reader,
                                    buf_size,
                                    mem_opts,
                                    len as usize,
                                ))
                            }
                            None => crate::body::stream(FileStream::new(reader, buf_size)),
                        }
                    };

                    #[cfg(not(feature = "mem-cache"))]
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

                    resp.headers_mut()
                        .insert(hyper::header::CONTENT_LENGTH, HeaderValue::from(len));
                    insert_raw(resp.headers_mut(), CONTENT_TYPE, content_type);
                    insert_raw(
                        resp.headers_mut(),
                        ACCEPT_RANGES,
                        ACCEPT_RANGES_BYTES.clone(),
                    );

                    if let Some(last_modified) = modified {
                        resp.headers_mut().typed_insert(last_modified);
                    }
                    if let Some(hv) = etag_value {
                        insert_raw(resp.headers_mut(), ETAG, hv.clone());
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
