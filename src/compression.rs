// Compression handler that compress the body of a response.
// -> Part of the file is borrowed from https://github.com/seanmonstar/warp/pull/513

use async_compression::tokio::bufread::{BrotliEncoder, DeflateEncoder, GzipEncoder};
use bytes::Bytes;
use futures::Stream;
use headers::{AcceptEncoding, ContentCoding, ContentType, HeaderMap, HeaderMapExt};
use http::header::HeaderValue;
use hyper::{
    header::{CONTENT_ENCODING, CONTENT_LENGTH},
    Body, Response,
};
use pin_project::pin_project;
use std::convert::TryFrom;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_util::io::{ReaderStream, StreamReader};

use crate::error::Result;

/// Contains a fixed list of common text-based MIME types in order to apply compression.
pub const TEXT_MIME_TYPES: [&str; 16] = [
    "text/html",
    "text/css",
    "text/javascript",
    "text/xml",
    "text/plain",
    "text/x-component",
    "application/javascript",
    "application/x-javascript",
    "application/json",
    "application/xml",
    "application/rss+xml",
    "application/atom+xml",
    "font/truetype",
    "font/opentype",
    "application/vnd.ms-fontobject",
    "image/svg+xml",
];

/// Create a wrapping handler that compresses the Body of a [`Response`](hyper::Response)
/// using `gzip`, `deflate` or `brotli` if is specified in the `Accept-Encoding` header, adding
/// `content-encoding: <coding>` to the Response's [`HeaderMap`](hyper::HeaderMap)
/// It also provides the ability to apply compression for text-based MIME types only.
pub fn auto(headers: &HeaderMap<HeaderValue>, resp: Response<Body>) -> Result<Response<Body>> {
    // Skip compression for non-text-based MIME types
    if let Some(content_type) = resp.headers().typed_get::<ContentType>() {
        let content_type = content_type.to_string();
        if !TEXT_MIME_TYPES.iter().any(|h| *h == content_type) {
            return Ok(resp);
        }
    }

    if let Some(ref accept_encoding) = headers.typed_get::<AcceptEncoding>() {
        if let Some(encoding) = accept_encoding.prefered_encoding() {
            if encoding == ContentCoding::GZIP {
                let (head, body) = resp.into_parts();
                return Ok(gzip(head, body.into()));
            }
            if encoding == ContentCoding::DEFLATE {
                let (head, body) = resp.into_parts();
                return Ok(deflate(head, body.into()));
            }
            if encoding == ContentCoding::BROTLI {
                let (head, body) = resp.into_parts();
                return Ok(brotli(head, body.into()));
            }
        }
    }

    Ok(resp)
}

/// Create a wrapping handler that compresses the Body of a [`Response`](hyper::Response)
/// using gzip, adding `content-encoding: gzip` to the Response's [`HeaderMap`](hyper::HeaderMap)
pub fn gzip(
    mut head: http::response::Parts,
    body: CompressableBody<Body, hyper::Error>,
) -> Response<Body> {
    let body = Body::wrap_stream(ReaderStream::new(GzipEncoder::new(StreamReader::new(body))));
    let header = create_encoding_header(head.headers.remove(CONTENT_ENCODING), ContentCoding::GZIP);
    head.headers.remove(CONTENT_LENGTH);
    head.headers.append(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Create a wrapping handler that compresses the Body of a [`Response`](hyper::Response)
/// using deflate, adding `content-encoding: deflate` to the Response's [`HeaderMap`](hyper::HeaderMap)
pub fn deflate(
    mut head: http::response::Parts,
    body: CompressableBody<Body, hyper::Error>,
) -> Response<Body> {
    let body = Body::wrap_stream(ReaderStream::new(DeflateEncoder::new(StreamReader::new(
        body,
    ))));
    let header = create_encoding_header(
        head.headers.remove(CONTENT_ENCODING),
        ContentCoding::DEFLATE,
    );
    head.headers.remove(CONTENT_LENGTH);
    head.headers.append(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Create a wrapping handler that compresses the Body of a [`Response`](hyper::Response)
/// using brotli, adding `content-encoding: br` to the Response's [`HeaderMap`](hyper::HeaderMap)
pub fn brotli(
    mut head: http::response::Parts,
    body: CompressableBody<Body, hyper::Error>,
) -> Response<Body> {
    let body = Body::wrap_stream(ReaderStream::new(BrotliEncoder::new(StreamReader::new(
        body,
    ))));
    let header =
        create_encoding_header(head.headers.remove(CONTENT_ENCODING), ContentCoding::BROTLI);
    head.headers.remove(CONTENT_LENGTH);
    head.headers.append(CONTENT_ENCODING, header);
    Response::from_parts(head, body)
}

/// Given an optional existing encoding header, appends to the existing or creates a new one.
fn create_encoding_header(existing: Option<HeaderValue>, coding: ContentCoding) -> HeaderValue {
    if let Some(val) = existing {
        if let Ok(str_val) = val.to_str() {
            return HeaderValue::try_from(&format!("{}, {}", str_val, coding.to_string()))
                .unwrap_or_else(|_| coding.into());
        }
    }
    coding.into()
}
/// A wrapper around any type that implements [`Stream`](futures::Stream) to be
/// compatible with async_compression's Stream based encoders.
#[pin_project]
#[derive(Debug)]
pub struct CompressableBody<S, E>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: std::error::Error,
{
    #[pin]
    pub body: S,
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
        // TODO: Use `.map_err()` (https://github.com/rust-lang/rust/issues/63514) once it is stabilized
        S::poll_next(pin.body, ctx)
            .map(|err| err.map(|res| res.map_err(|_| Error::from(ErrorKind::InvalidData))))
    }
}

impl From<Body> for CompressableBody<Body, hyper::Error> {
    fn from(body: Body) -> Self {
        CompressableBody { body }
    }
}
