// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to transition files into HTTP responses.
//!

use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, Header, HeaderMapExt, LastModified,
    Range,
};
use hyper::{Body, Response, StatusCode};
use std::fs::{File, Metadata};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::Bound;
use std::path::PathBuf;

use crate::conditional_headers::{ConditionalBody, ConditionalHeaders};
use crate::fs::stream::{FileStream, optimal_buf_size};

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

    for (start, end) in range.iter() {
        tracing::trace!("range request received, {:?}-{:?}-{}", start, end, max_len);
        match normalize_byte_range(start, end, max_len)? {
            Some((start, end)) => {
                tracing::trace!("range request to return: {}-{}/{}", start, end, max_len);
                return Ok((start, end));
            }
            None => tracing::trace!("unsatisfiable byte range for length {max_len}"),
        }
    }

    // NOTE: default to `BadRangeError` in case of wrong `Range` bytes format.
    // Special case: suffix ranges (bytes=-N) where N > file size are valid per
    // RFC 9110 section 14.1.2 and should return the entire file (200), but
    // headers 0.4 `satisfiable_ranges(len)` filters them out. Inspect the
    // original header on this cold path so out-of-bounds first-byte ranges
    // (for example bytes=5000-) remain unsatisfiable instead of being mistaken
    // for an oversized suffix range.
    if has_oversized_suffix_range(&range, max_len) {
        tracing::trace!(
            "suffix range exceeds file size, returning full content: 0-{}/{}",
            max_len,
            max_len
        );
        Ok((0, max_len))
    } else {
        Err(BadRangeError)
    }
}

fn normalize_byte_range(
    start: Bound<u64>,
    end: Bound<u64>,
    max_len: u64,
) -> Result<Option<(u64, u64)>, BadRangeError> {
    match (start, end) {
        (Bound::Included(start), Bound::Included(end)) => {
            if start > end {
                return Err(BadRangeError);
            }
            if start >= max_len {
                return Ok(None);
            }
            let end = if end >= max_len { max_len } else { end + 1 };
            Ok((start < end).then_some((start, end)))
        }
        (Bound::Included(start), Bound::Unbounded) => {
            Ok((start < max_len).then_some((start, max_len)))
        }
        (Bound::Unbounded, Bound::Included(suffix_len)) => {
            if suffix_len == 0 || max_len == 0 {
                return Ok(None);
            }
            if suffix_len >= max_len {
                Ok(Some((0, max_len)))
            } else {
                Ok(Some((max_len - suffix_len, max_len)))
            }
        }
        _ => Err(BadRangeError),
    }
}

#[cold]
fn has_oversized_suffix_range(range: &Range, max_len: u64) -> bool {
    if max_len == 0 {
        return false;
    }

    let mut values = Vec::with_capacity(1);
    range.encode(&mut values);
    values
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.strip_prefix("bytes="))
        .flat_map(|specs| specs.split(','))
        .any(|spec| suffix_len_exceeds(spec.trim(), max_len))
}

fn suffix_len_exceeds(spec: &str, max_len: u64) -> bool {
    let Some(digits) = spec.strip_prefix('-') else {
        return false;
    };
    if digits.is_empty() {
        return false;
    }

    let mut value = 0_u64;
    let mut non_zero = false;
    for byte in digits.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
        let digit = u64::from(byte - b'0');
        non_zero |= digit != 0;
        let Some(next) = value.checked_mul(10).and_then(|v| v.checked_add(digit)) else {
            return true;
        };
        value = next;
        if value > max_len {
            return true;
        }
    }

    non_zero && value > max_len
}

#[cfg(test)]
mod tests {
    use headers::{HeaderMap, HeaderMapExt, Range};

    use super::bytes_range;

    fn range(spec: &str) -> Option<Range> {
        let mut map = HeaderMap::new();
        map.insert(
            http::header::RANGE,
            format!("bytes={spec}").parse().unwrap(),
        );
        map.typed_get::<Range>()
    }

    #[test]
    fn no_range_returns_full_file() {
        assert_eq!(bytes_range(None, 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn inclusive_range_within_bounds() {
        assert_eq!(bytes_range(range("0-499"), 1000).unwrap(), (0, 500));
    }

    #[test]
    fn inclusive_range_to_last_byte() {
        assert_eq!(bytes_range(range("500-999"), 1000).unwrap(), (500, 1000));
    }

    #[test]
    fn suffix_range_within_file() {
        assert_eq!(bytes_range(range("-200"), 1000).unwrap(), (800, 1000));
    }

    #[test]
    fn suffix_range_larger_than_file_returns_full() {
        assert_eq!(bytes_range(range("-2000"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn enormous_suffix_range_returns_full() {
        assert_eq!(
            bytes_range(range("-18446744073709551616"), 1000).unwrap(),
            (0, 1000)
        );
    }

    #[test]
    fn open_ended_range_from_offset() {
        assert_eq!(bytes_range(range("100-"), 1000).unwrap(), (100, 1000));
    }

    #[test]
    fn range_start_equals_end_is_single_byte() {
        assert_eq!(bytes_range(range("5-5"), 1000).unwrap(), (5, 6));
    }

    #[test]
    fn range_start_greater_than_end_is_error() {
        assert!(bytes_range(range("100-50"), 1000).is_err());
    }

    #[test]
    fn range_start_beyond_file_size_is_error() {
        assert!(bytes_range(range("2000-3000"), 1000).is_err());
    }

    #[test]
    fn open_ended_range_starting_at_file_size_is_error() {
        assert!(bytes_range(range("1000-"), 1000).is_err());
    }

    #[test]
    fn out_of_bounds_first_byte_ranges_are_errors() {
        assert!(bytes_range(range("5000-"), 100).is_err());
        assert!(bytes_range(range("5000-5999"), 100).is_err());
        assert!(bytes_range(range("100-199"), 100).is_err());
    }

    #[test]
    fn range_end_beyond_file_size_is_clamped() {
        assert_eq!(bytes_range(range("50-999"), 100).unwrap(), (50, 100));
    }

    #[test]
    fn huge_range_end_does_not_overflow() {
        assert_eq!(
            bytes_range(range("0-18446744073709551615"), 100).unwrap(),
            (0, 100)
        );
    }

    #[test]
    fn invalid_empty_range_is_error() {
        assert!(bytes_range(range("-"), 100).is_err());
    }

    #[test]
    fn range_on_zero_byte_file_is_unsatisfiable() {
        assert!(bytes_range(range("0-0"), 0).is_err());
        assert!(bytes_range(range("-1"), 0).is_err());
        assert!(bytes_range(range("0-"), 0).is_err());
    }

    #[test]
    fn later_satisfiable_range_is_used() {
        assert_eq!(bytes_range(range("200-300,0-9"), 100).unwrap(), (0, 10));
    }

    #[test]
    fn suffix_range_equal_to_file_size_returns_full() {
        assert_eq!(bytes_range(range("-1000"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn suffix_range_of_one_byte() {
        assert_eq!(bytes_range(range("-1"), 1000).unwrap(), (999, 1000));
    }

    #[test]
    fn zero_length_suffix_is_unsatisfiable() {
        assert!(bytes_range(range("-0"), 1000).is_err());
    }

    #[test]
    fn leading_zeros_in_suffix_are_parsed() {
        assert_eq!(bytes_range(range("-00200"), 1000).unwrap(), (800, 1000));
        assert_eq!(bytes_range(range("-02000"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn full_file_range_returns_full() {
        assert_eq!(bytes_range(range("0-999"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn multi_range_first_satisfiable_wins() {
        assert_eq!(bytes_range(range("0-9,50-59"), 100).unwrap(), (0, 10));
    }

    #[test]
    fn multi_range_with_oversized_suffix_picks_first_valid() {
        assert_eq!(bytes_range(range("10-19,-99999"), 100).unwrap(), (10, 20));
    }

    #[test]
    fn range_at_u64_boundary_file_size() {
        assert_eq!(bytes_range(range("0-"), u64::MAX).unwrap(), (0, u64::MAX));
    }
}
