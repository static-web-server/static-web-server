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

use bytes::Bytes;
use futures_util::Stream;
use headers::{ContentType, HeaderMap, HeaderMapExt, HeaderValue};
use hyper::{
    header::{CONTENT_ENCODING, CONTENT_LENGTH},
    Body, Method, Request, Response, StatusCode,
};
use lazy_static::lazy_static;
use mime_guess::{mime, Mime};
use pin_project::pin_project;
use std::collections::HashSet;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_util::io::{ReaderStream, StreamReader};

use crate::{
    error_page,
    handler::RequestHandlerOpts,
    headers_ext::{AcceptEncoding, ContentCoding},
    http_ext::MethodExt,
    settings::CompressionLevel,
    Error, Result,
};

lazy_static! {
    /// Contains a fixed list of common text-based MIME types that aren't recognizable in a generic way.
    static ref TEXT_MIME_TYPES: HashSet<&'static str> = [
        "application/rtf",
        "application/javascript",
        "application/json",
        "application/xml",
        "font/ttf",
        "application/font-sfnt",
        "application/vnd.ms-fontobject",
        "application/wasm",
    ].into_iter().collect();
}

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
    server_info!(
        "auto compression: enabled={enabled}, formats={}, compression level={level:?}",
        FORMATS.join(",")
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
    resp.headers_mut().insert(
        hyper::header::VARY,
        HeaderValue::from_name(hyper::header::ACCEPT_ENCODING),
    );

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
        if let Some(content_type) = resp.headers().typed_get::<ContentType>() {
            if !is_text(Mime::from(content_type)) {
                return Ok(resp);
            }
        }

        #[cfg(any(feature = "compression", feature = "compression-gzip"))]
        if encoding == ContentCoding::GZIP {
            let (head, body) = resp.into_parts();
            return Ok(gzip(head, body.into(), level));
        }

        #[cfg(any(feature = "compression", feature = "compression-deflate"))]
        if encoding == ContentCoding::DEFLATE {
            let (head, body) = resp.into_parts();
            return Ok(deflate(head, body.into(), level));
        }

        #[cfg(any(feature = "compression", feature = "compression-brotli"))]
        if encoding == ContentCoding::BROTLI {
            let (head, body) = resp.into_parts();
            return Ok(brotli(head, body.into(), level));
        }

        #[cfg(any(feature = "compression", feature = "compression-zstd"))]
        if encoding == ContentCoding::ZSTD {
            let (head, body) = resp.into_parts();
            return Ok(zstd(head, body.into(), level));
        }

        tracing::trace!("no compression feature matched the preferred encoding, probably not enabled or unsupported");
    }

    Ok(resp)
}

/// Checks whether the MIME type corresponds to any of the known text types.
fn is_text(mime: Mime) -> bool {
    mime.type_() == mime::TEXT
        || mime
            .suffix()
            .is_some_and(|suffix| suffix == mime::XML || suffix == mime::JSON)
        || TEXT_MIME_TYPES.contains(mime.essence_str())
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
    body: CompressableBody<Body, hyper::Error>,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 4;

    tracing::trace!("compressing response body on the fly using GZIP");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = Body::wrap_stream(ReaderStream::new(GzipEncoder::with_quality(
        StreamReader::new(body),
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
    body: CompressableBody<Body, hyper::Error>,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 4;

    tracing::trace!("compressing response body on the fly using DEFLATE");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = Body::wrap_stream(ReaderStream::new(DeflateEncoder::with_quality(
        StreamReader::new(body),
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
    body: CompressableBody<Body, hyper::Error>,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 4;

    tracing::trace!("compressing response body on the fly using BROTLI");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = Body::wrap_stream(ReaderStream::new(BrotliEncoder::with_quality(
        StreamReader::new(body),
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
    body: CompressableBody<Body, hyper::Error>,
    level: CompressionLevel,
) -> Response<Body> {
    const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

    tracing::trace!("compressing response body on the fly using ZSTD");

    let level = level.into_algorithm_level(DEFAULT_COMPRESSION_LEVEL);
    let body = Body::wrap_stream(ReaderStream::new(ZstdEncoder::with_quality(
        StreamReader::new(body),
        level,
    )));
    let header = create_encoding_header(head.headers.remove(CONTENT_ENCODING), ContentCoding::ZSTD);
    head.headers.remove(CONTENT_LENGTH);
    head.headers.insert(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Given an optional existing encoding header, appends to the existing or creates a new one.
pub fn create_encoding_header(existing: Option<HeaderValue>, coding: ContentCoding) -> HeaderValue {
    if let Some(val) = existing {
        if let Ok(str_val) = val.to_str() {
            return HeaderValue::from_str(&[str_val, ", ", coding.as_str()].concat())
                .unwrap_or_else(|_| coding.into());
        }
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

/// A wrapper around any type that implements [`Stream`](futures_util::Stream) to be
/// compatible with async_compression's `Stream` based encoders.
#[pin_project]
#[derive(Debug)]
pub struct CompressableBody<S, E>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: std::error::Error,
{
    #[pin]
    body: S,
}

impl<S, E> Stream for CompressableBody<S, E>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: std::error::Error,
{
    type Item = std::io::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use std::io::{Error, ErrorKind};

        let pin = self.project();
        S::poll_next(pin.body, ctx).map_err(|_| Error::from(ErrorKind::InvalidData))
    }
}

impl From<Body> for CompressableBody<Body, hyper::Error> {
    #[inline(always)]
    fn from(body: Body) -> Self {
        CompressableBody { body }
    }
}
