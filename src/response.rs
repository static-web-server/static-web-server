// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to transition files into HTTP responses.
//!

use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMapExt, LastModified, Range,
};
use hyper::{Response, StatusCode};
use std::fs::{File, Metadata};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::Bound;
use std::path::PathBuf;

use crate::body::Body;
use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::fs::stream::{FileStream, optimal_buf_size};

#[cfg(feature = "mem-cache")]
use crate::mem_cache::{
    cache::{MemCacheOpts, MemFileTempOpts},
    stream::MemCacheFileStream,
};

/// It converts a file object into a corresponding HTTP response or
/// returns an error holding an HTTP status code otherwise.
pub(crate) fn response_body(
    mut file: File,
    path: &PathBuf,
    meta: &Metadata,
    conditionals: ConditionalHeaders,
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
                    let mut resp = Response::new(crate::body::empty());
                    *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                    resp.headers_mut()
                        .typed_insert(ContentRange::unsatisfied_bytes(len));
                    Ok(resp)
                })
        }
    }
}

#[derive(Debug)]
pub(crate) struct BadRangeError;

/// It handles the `Range` header returning the corresponding start/end-range bytes
/// or returns an error for bad ranges otherwise.
pub(crate) fn bytes_range(range: Option<Range>, max_len: u64) -> Result<(u64, u64), BadRangeError> {
    let range = if let Some(range) = range {
        range
    } else {
        return Ok((0, max_len));
    };

    range
        .satisfiable_ranges(max_len)
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
                _ => return Err(BadRangeError),
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
        // NOTE: default to `BadRangeError` in case of wrong `Range` bytes format.
        // Special case: suffix ranges (bytes=-N) where N > file size are valid per
        // RFC 9110 §14.1.2 and should return the entire file (200), but headers 0.4
        // `satisfiable_ranges(len)` filters them out. Detect this by re-checking with
        // `u64::MAX`. If that yields a result, then it was a valid-but-oversized suffix range.
        .unwrap_or_else(|| {
            if range.satisfiable_ranges(u64::MAX).next().is_some() {
                tracing::trace!(
                    "suffix range exceeds file size, returning full content: 0-{}/{}",
                    max_len,
                    max_len
                );
                Ok((0, max_len))
            } else {
                Err(BadRangeError)
            }
        })
}

#[cfg(test)]
mod tests {
    use headers::{HeaderMap, HeaderMapExt, Range};

    use super::bytes_range;

    fn range(s: &str) -> Option<Range> {
        let mut map = HeaderMap::new();
        map.insert(http::header::RANGE, format!("bytes={s}").parse().unwrap());
        map.typed_get::<Range>()
    }

    #[test]
    fn no_range_returns_full_file() {
        assert_eq!(bytes_range(None, 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn inclusive_range_within_bounds() {
        // bytes=0-499 of 1000-byte file → (0, 500)
        assert_eq!(bytes_range(range("0-499"), 1000).unwrap(), (0, 500));
    }

    #[test]
    fn inclusive_range_to_last_byte() {
        // bytes=500-999 of 1000-byte file → (500, 1000)
        assert_eq!(bytes_range(range("500-999"), 1000).unwrap(), (500, 1000));
    }

    #[test]
    fn suffix_range_within_file() {
        // bytes=-200 of 1000-byte file → last 200 bytes = (800, 1000)
        assert_eq!(bytes_range(range("-200"), 1000).unwrap(), (800, 1000));
    }

    #[test]
    fn suffix_range_larger_than_file_returns_full() {
        // bytes=-2000 of 1000-byte file: suffix exceeds file size → return entire file
        assert_eq!(bytes_range(range("-2000"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn open_ended_range_from_offset() {
        // bytes=100- of 1000-byte file → (100, 1000)
        assert_eq!(bytes_range(range("100-"), 1000).unwrap(), (100, 1000));
    }

    #[test]
    fn range_start_equals_end_is_single_byte() {
        // bytes=5-5 of 1000-byte file → (5, 6)
        assert_eq!(bytes_range(range("5-5"), 1000).unwrap(), (5, 6));
    }

    #[test]
    fn range_start_greater_than_end_is_error() {
        // bytes=100-50 → invalid
        assert!(bytes_range(range("100-50"), 1000).is_err());
    }

    #[test]
    fn range_start_beyond_file_size_is_error() {
        // bytes=2000-3000 of 1000-byte file → unsatisfiable
        assert!(bytes_range(range("2000-3000"), 1000).is_err());
    }
}
