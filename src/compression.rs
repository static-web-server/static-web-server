// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Auto-compression module to compress responses body.
//!

// Part of the file is borrowed from <https://github.com/seanmonstar/warp/pull/513>*

#[cfg(any(feature = "compression", feature = "compression-brotli"))]
use async_compression::tokio::bufread::BrotliEncoder;
#[cfg(any(feature = "compression", feature = "compression-deflate"))]
use async_compression::tokio::bufread::DeflateEncoder;
#[cfg(any(feature = "compression", feature = "compression-gzip"))]
use async_compression::tokio::bufread::GzipEncoder;
#[cfg(any(feature = "compression", feature = "compression-zstd"))]
use async_compression::tokio::bufread::ZstdEncoder;

use headers::{ContentType, HeaderMap, HeaderMapExt, HeaderValue};
use http_body_util::BodyExt as _;
use hyper::{
    Method, Request, Response, StatusCode,
    header::{CONTENT_ENCODING, CONTENT_LENGTH},
};
use mime_guess::Mime;
use tokio_util::io::{ReaderStream, StreamReader};

use crate::body::Body;
use crate::error_page;
use crate::exts::headers::{AcceptEncoding, ContentCoding};
use crate::exts::http::{MethodExt, append_vary_accept_encoding};
use crate::exts::mime::MimeExt;
use crate::handler::RequestHandlerOpts;
use crate::settings::CompressionLevel;
use crate::{Error, Result};

/// Minimum response body size in bytes below which dynamic compression is skipped.
const MIN_COMPRESS_SIZE: usize = 200;

/// List of encodings that can be handled given enabled features.
const AVAILABLE_ENCODINGS: &[ContentCoding] = &[
    #[cfg(any(feature = "compression", feature = "compression-deflate"))]
    ContentCoding::DEFLATE,
    #[cfg(any(feature = "compression", feature = "compression-gzip"))]
    ContentCoding::GZIP,
    #[cfg(any(feature = "compression", feature = "compression-brotli"))]
    ContentCoding::BROTLI,
    #[cfg(any(feature = "compression", feature = "compression-zstd"))]
    ContentCoding::ZSTD,
];

/// Initializes dynamic compression.
pub fn init(enabled: bool, level: CompressionLevel, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.compression = enabled;
    handler_opts.compression_level = level;

    const FORMATS: &[&str] = &[
        #[cfg(any(feature = "compression", feature = "compression-deflate"))]
        "deflate",
        #[cfg(any(feature = "compression", feature = "compression-gzip"))]
        "gzip",
        #[cfg(any(feature = "compression", feature = "compression-brotli"))]
        "brotli",
        #[cfg(any(feature = "compression", feature = "compression-zstd"))]
        "zstd",
    ];
    tracing::info!(
        enabled,
        formats = %FORMATS.join(","),
        compression_level = ?level,
        "auto compression"
    );
}

/// Post-processing to dynamically compress the response if necessary.
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if !opts.compression {
        return Ok(resp);
    }

    let is_precompressed = resp.headers().get(CONTENT_ENCODING).is_some();
    if is_precompressed {
        return Ok(resp);
    }

    // Compression content encoding varies so use a `Vary` header
    append_vary_accept_encoding(&mut resp);

    // Auto compression based on the `Accept-Encoding` header
    match auto(req.method(), req.headers(), opts.compression_level, resp) {
        Ok(resp) => Ok(resp),
        Err(err) => {
            tracing::error!("error during body compression: {:?}", err);
            error_page::error_response(
                req.uri(),
                req.method(),
                &StatusCode::INTERNAL_SERVER_ERROR,
                &opts.page404,
                &opts.page50x,
            )
        }
    }
}

/// Create a wrapping handler that compresses the Body of a [`hyper::Response`]
/// using gzip, `deflate`, `brotli` or `zstd` if is specified in the `Accept-Encoding` header, adding
/// `content-encoding: <coding>` to the Response's [`HeaderMap`].
/// It also provides the ability to apply compression for text-based MIME types only.
pub fn auto(
    method: &Method,
    headers: &HeaderMap<HeaderValue>,
    level: CompressionLevel,
    resp: Response<Body>,
) -> Result<Response<Body>> {
    // Skip compression for HEAD and OPTIONS request methods
    if method.is_head() || method.is_options() {
        return Ok(resp);
    }

    // Compress response based on Accept-Encoding header
    if let Some(encoding) = get_preferred_encoding(headers) {
        tracing::trace!(
            "preferred encoding selected from the accept-encoding header: {:?}",
            encoding
        );

        // Skip compression for non-text-based MIME types
        if let Some(content_type) = resp.headers().typed_get::<ContentType>()
            && !Mime::from(content_type).is_compressible()
        {
            return Ok(resp);
        }

        // Skip compression for responses below the minimum size threshold.
        // Tiny payloads gain no benefit and the compression overhead can
        // make them larger than the original.
        if let Some(content_length) = resp
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            && content_length < MIN_COMPRESS_SIZE
        {
            tracing::trace!(
                "skipping compression: content-length ({content_length}) below minimum ({MIN_COMPRESS_SIZE})",
            );
            return Ok(resp);
        }

        #[cfg(any(feature = "compression", feature = "compression-gzip"))]
        if encoding == ContentCoding::GZIP {
            let (head, body) = resp.into_parts();
            return Ok(gzip(head, body, level));
        }

        #[cfg(any(feature = "compression", feature = "compression-deflate"))]
        if encoding == ContentCoding::DEFLATE {
            let (head, body) = resp.into_parts();
            return Ok(deflate(head, body, level));
        }

        #[cfg(any(feature = "compression", feature = "compression-brotli"))]
        if encoding == ContentCoding::BROTLI {
            let (head, body) = resp.into_parts();
            return Ok(brotli(head, body, level));
        }

        #[cfg(any(feature = "compression", feature = "compression-zstd"))]
        if encoding == ContentCoding::ZSTD {
            let (head, body) = resp.into_parts();
            return Ok(zstd(head, body, level));
        }

        tracing::trace!(
            "no compression feature matched the preferred encoding, probably not enabled or unsupported"
        );
    }

    Ok(resp)
}

/// Create a wrapping handler that compresses the Body of a [`Response`].
/// using gzip, adding `content-encoding: gzip` to the Response's [`HeaderMap`].
#[cfg(any(feature = "compression", feature = "compression-gzip"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "compression", feature = "compression-gzip")))
)]
pub fn gzip(
    mut head: http::response::Parts,
    body: Body,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 4;

    tracing::trace!("compressing response body on the fly using GZIP");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = crate::body::stream(ReaderStream::new(GzipEncoder::with_quality(
        StreamReader::new(body.into_data_stream()),
        level,
    )));
    let header = create_encoding_header(head.headers.remove(CONTENT_ENCODING), ContentCoding::GZIP);
    head.headers.remove(CONTENT_LENGTH);
    head.headers.insert(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Create a wrapping handler that compresses the Body of a [`Response`].
/// using deflate, adding `content-encoding: deflate` to the Response's [`HeaderMap`].
#[cfg(any(feature = "compression", feature = "compression-deflate"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "compression", feature = "compression-deflate")))
)]
pub fn deflate(
    mut head: http::response::Parts,
    body: Body,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 4;

    tracing::trace!("compressing response body on the fly using DEFLATE");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = crate::body::stream(ReaderStream::new(DeflateEncoder::with_quality(
        StreamReader::new(body.into_data_stream()),
        level,
    )));
    let header = create_encoding_header(
        head.headers.remove(CONTENT_ENCODING),
        ContentCoding::DEFLATE,
    );
    head.headers.remove(CONTENT_LENGTH);
    head.headers.insert(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Create a wrapping handler that compresses the Body of a [`Response`].
/// using brotli, adding `content-encoding: br` to the Response's [`HeaderMap`].
#[cfg(any(feature = "compression", feature = "compression-brotli"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "compression", feature = "compression-brotli")))
)]
pub fn brotli(
    mut head: http::response::Parts,
    body: Body,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 4;

    tracing::trace!("compressing response body on the fly using BROTLI");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = crate::body::stream(ReaderStream::new(BrotliEncoder::with_quality(
        StreamReader::new(body.into_data_stream()),
        level,
    )));
    let header =
        create_encoding_header(head.headers.remove(CONTENT_ENCODING), ContentCoding::BROTLI);
    head.headers.remove(CONTENT_LENGTH);
    head.headers.insert(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Create a wrapping handler that compresses the Body of a [`Response`].
/// using zstd, adding `content-encoding: zstd` to the Response's [`HeaderMap`].
#[cfg(any(feature = "compression", feature = "compression-zstd"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "compression", feature = "compression-zstd")))
)]
pub fn zstd(
    mut head: http::response::Parts,
    body: Body,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

    tracing::trace!("compressing response body on the fly using ZSTD");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = crate::body::stream(ReaderStream::new(ZstdEncoder::with_quality(
        StreamReader::new(body.into_data_stream()),
        level,
    )));
    let header = create_encoding_header(head.headers.remove(CONTENT_ENCODING), ContentCoding::ZSTD);
    head.headers.remove(CONTENT_LENGTH);
    head.headers.insert(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Given an optional existing encoding header, appends to the existing or creates a new one.
pub fn create_encoding_header(existing: Option<HeaderValue>, coding: ContentCoding) -> HeaderValue {
    if let Some(val) = existing
        && let Ok(str_val) = val.to_str()
    {
        return HeaderValue::from_str(&[str_val, ", ", coding.as_str()].concat())
            .unwrap_or_else(|_| coding.into());
    }
    coding.into()
}

/// Try to get the preferred `content-encoding` via the `accept-encoding` header.
#[inline(always)]
pub fn get_preferred_encoding(headers: &HeaderMap<HeaderValue>) -> Option<ContentCoding> {
    if let Some(ref accept_encoding) = headers.typed_get::<AcceptEncoding>() {
        tracing::trace!("request with accept-encoding header: {:?}", accept_encoding);

        for encoding in accept_encoding.sorted_encodings() {
            if AVAILABLE_ENCODINGS.contains(&encoding) {
                return Some(encoding);
            }
        }
    }
    None
}

/// Get the `content-encodings` via the `accept-encoding` header.
#[inline(always)]
pub fn get_encodings(headers: &HeaderMap<HeaderValue>) -> Vec<ContentCoding> {
    if let Some(ref accept_encoding) = headers.typed_get::<AcceptEncoding>() {
        tracing::trace!("request with accept-encoding header: {:?}", accept_encoding);

        return accept_encoding
            .sorted_encodings()
            .filter(|encoding| AVAILABLE_ENCODINGS.contains(encoding))
            .collect::<Vec<_>>();
    }
    vec![]
}

#[cfg(test)]
#[cfg(any(feature = "compression", feature = "compression-gzip"))]
mod tests {
    use super::*;
    use crate::body;
    use crate::settings::CompressionLevel;
    use http::header::{ACCEPT_ENCODING, CONTENT_TYPE};
    use hyper::{Method, Response};

    /// Build a `Response<Body>` with a text content-type header,
    /// a content-length header, and a body of `size` bytes.
    fn text_response_with_size(size: usize) -> Response<Body> {
        let body = body::full(vec![b'x'; size]);
        let mut resp = Response::new(body);
        resp.headers_mut()
            .insert(CONTENT_TYPE, "text/html".parse().unwrap());
        resp.headers_mut()
            .insert(CONTENT_LENGTH, size.to_string().parse().unwrap());
        resp
    }

    /// Build a `Response<Body>` without any `Content-Length` header.
    fn text_response_without_length() -> Response<Body> {
        let body = body::full(b"hello world".as_slice());
        let mut resp = Response::new(body);
        resp.headers_mut()
            .insert(CONTENT_TYPE, "text/html".parse().unwrap());
        resp
    }

    /// Build a simple GET `HeaderMap` with an Accept-Encoding: gzip header.
    fn accept_gzip_headers() -> HeaderMap<HeaderValue> {
        let mut h = HeaderMap::new();
        h.insert(ACCEPT_ENCODING, "gzip".parse().unwrap());
        h
    }

    // Minimum-size threshold tests

    #[test]
    fn small_response_below_threshold_is_not_compressed() {
        let resp = text_response_with_size(MIN_COMPRESS_SIZE - 1);
        let headers = accept_gzip_headers();
        let result = auto(&Method::GET, &headers, CompressionLevel::Default, resp).unwrap();
        // no content-encoding should be set
        assert!(
            result.headers().get(CONTENT_ENCODING).is_none(),
            "responses below {MIN_COMPRESS_SIZE} bytes must not be compressed"
        );
    }

    #[test]
    fn response_at_threshold_is_compressed() {
        let resp = text_response_with_size(MIN_COMPRESS_SIZE);
        let headers = accept_gzip_headers();
        let result = auto(&Method::GET, &headers, CompressionLevel::Default, resp).unwrap();
        assert!(
            result.headers().get(CONTENT_ENCODING).is_some(),
            "responses at exactly {MIN_COMPRESS_SIZE} bytes must be compressed"
        );
    }

    #[test]
    fn response_above_threshold_is_compressed() {
        let resp = text_response_with_size(MIN_COMPRESS_SIZE + 1);
        let headers = accept_gzip_headers();
        let result = auto(&Method::GET, &headers, CompressionLevel::Default, resp).unwrap();
        assert!(
            result.headers().get(CONTENT_ENCODING).is_some(),
            "responses above {MIN_COMPRESS_SIZE} bytes must be compressed"
        );
    }

    #[test]
    fn response_without_content_length_is_compressed() {
        let resp = text_response_without_length();
        let headers = accept_gzip_headers();
        let result = auto(&Method::GET, &headers, CompressionLevel::Default, resp).unwrap();
        assert!(
            result.headers().get(CONTENT_ENCODING).is_some(),
            "responses without Content-Length must still be compressed (safe default)"
        );
    }

    #[test]
    fn small_response_head_method_is_not_compressed() {
        let resp = text_response_with_size(MIN_COMPRESS_SIZE - 1);
        let headers = accept_gzip_headers();
        let result = auto(&Method::HEAD, &headers, CompressionLevel::Default, resp).unwrap();
        assert!(
            result.headers().get(CONTENT_ENCODING).is_none(),
            "HEAD requests are never compressed regardless of size"
        );
    }

    #[test]
    fn non_compressible_content_type_is_not_compressed() {
        let body = body::full(vec![b'x'; MIN_COMPRESS_SIZE + 100]);
        let mut resp = Response::new(body);
        resp.headers_mut()
            .insert(CONTENT_TYPE, "image/png".parse().unwrap());
        resp.headers_mut().insert(
            CONTENT_LENGTH,
            (MIN_COMPRESS_SIZE + 100).to_string().parse().unwrap(),
        );
        let headers = accept_gzip_headers();
        let result = auto(&Method::GET, &headers, CompressionLevel::Default, resp).unwrap();
        assert!(
            result.headers().get(CONTENT_ENCODING).is_none(),
            "non-compressible content-types are never compressed"
        );
    }
}
