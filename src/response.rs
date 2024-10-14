// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to transition files into HTTP responses.
//!

use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMapExt, LastModified, Range,
};
use hyper::{Body, Response, StatusCode};
use std::fs::{File, Metadata};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::Bound;
use std::path::PathBuf;

use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::fs::stream::{optimal_buf_size, FileStream};

#[cfg(feature = "experimental")]
use {
    crate::mem_cache::{
        cache::{MemCacheOpts, MemFileTempOpts},
        stream::MemCacheFileStream,
    },
    bytes::BytesMut,
};

/// It converts a file object into a corresponding HTTP response or
/// returns an error holding an HTTP status code otherwise.
pub(crate) fn response_body(
    mut file: File,
    path: &PathBuf,
    meta: &Metadata,
    conditionals: ConditionalHeaders,
    #[cfg(feature = "experimental")] memory_cache: Option<&MemCacheOpts>,
) -> Result<Response<Body>, StatusCode> {
    let mut len = meta.len();
    // If the file's modified time is the UNIX epoch, then it's likely not valid and should
    // not be included in the Last-Modified header to avoid cache revalidation issues.
    let modified = meta
        .modified()
        .ok()
        .filter(|&t| t != std::time::UNIX_EPOCH)
        .map(LastModified::from);

    match conditionals.check(modified) {
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

                    let mime = mime_guess::from_path(path).first_or_octet_stream();
                    let content_type = ContentType::from(mime);

                    // Add the file to the in-memory cache only under these conditions:
                    // - if the feature is enabled and
                    // - if the file size does not exceed the maximum permitted and
                    // - if the file is not found in the cache store
                    // TODO: make this a feature
                    #[cfg(feature = "experimental")]
                    let body = match memory_cache {
                        // Cache the file only if does not exceed the max size
                        Some(mem_cache_opts) if len <= mem_cache_opts.max_file_size => {
                            match path.to_str() {
                                Some(path_str) => {
                                    let content_type = content_type.clone();
                                    let file_path = path_str.to_owned();

                                    let mem_buf = Some(BytesMut::with_capacity(len as usize));
                                    let mem_opts = Some(MemFileTempOpts::new(
                                        file_path,
                                        content_type,
                                        modified,
                                    ));
                                    tracing::debug!(
                                        "preparing `{}` to be inserted in-memory cache store",
                                        path_str,
                                    );
                                    Body::wrap_stream(MemCacheFileStream {
                                        reader,
                                        buf_size,
                                        mem_opts,
                                        mem_buf,
                                    })
                                }
                                _ => Body::wrap_stream(FileStream { reader, buf_size }),
                            }
                        }
                        _ => Body::wrap_stream(FileStream { reader, buf_size }),
                    };

                    #[cfg(not(feature = "experimental"))]
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
                    resp.headers_mut().typed_insert(content_type);
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

pub(crate) struct BadRangeError;

/// It handles the `Range` header returning the corresponding start/end-range bytes
/// or returns an error for bad ranges otherwise.
pub(crate) fn bytes_range(range: Option<Range>, max_len: u64) -> Result<(u64, u64), BadRangeError> {
    let range = if let Some(range) = range {
        range
    } else {
        return Ok((0, max_len));
    };

    let resp = range
        .iter()
        .map(|(start, end)| {
            tracing::trace!("range request received, {:?}-{:?}-{}", start, end, max_len);

            let (start, end) = match (start, end) {
                (Bound::Unbounded, Bound::Unbounded) => (0, max_len),
                (Bound::Included(a), Bound::Included(b)) => {
                    // `start` can not be greater than `end`
                    if a > b {
                        return Err(BadRangeError);
                    }
                    // For the special case where b == the file size
                    (a, if b == max_len { b } else { b + 1 })
                }
                (Bound::Included(a), Bound::Unbounded) => (a, max_len),
                (Bound::Unbounded, Bound::Included(b)) => {
                    if b > max_len {
                        // `Range` request out of bounds, return only what's available
                        tracing::trace!("unsatisfiable byte range: -{}/{}", b, max_len);
                        tracing::trace!("returning only what's available: 0-{}", max_len);
                        (0, max_len)
                    } else {
                        (max_len - b, max_len)
                    }
                }
                _ => unreachable!(),
            };

            if start < end && end <= max_len {
                tracing::trace!("range request to return: {}-{}/{}", start, end, max_len);
                return Ok((start, end));
            }

            tracing::trace!("unsatisfiable byte range: {}-{}/{}", start, end, max_len);

            if start < end && start <= max_len {
                // `Range` request out of bounds, return only what's available
                tracing::trace!(
                    "returning only what's available: {}-{}/{}",
                    start,
                    max_len,
                    max_len
                );
                return Ok((start, max_len));
            }

            Err(BadRangeError)
        })
        .next()
        // NOTE: default to `BadRangeError` in case of wrong `Range` bytes format
        .unwrap_or(Err(BadRangeError));

    resp
}
