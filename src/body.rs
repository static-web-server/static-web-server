// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Unified HTTP response body type and constructor helpers.
//!
//! This module provides a single concrete body type alias and a small set of
//! constructor functions.

use bytes::Bytes;
use http_body::Frame;
use http_body_util::{BodyExt, Empty, Full, StreamBody, combinators::BoxBody};
use std::io;

/// Unified response body type used throughout the server.
///
/// A type-erased boxed body backed by [`http_body_util::combinators::BoxBody`]
/// with [`bytes::Bytes`] data frames and [`io::Error`] errors.
///
/// This single type covers all body variants used by the server:
/// - Empty bodies (for HEAD responses, OPTIONS, redirects)
/// - In-memory byte buffers (for generated HTML, Prometheus metrics, health checks)
/// - File streams ([`crate::fs::stream::FileStream`])
/// - Compressed streams (gzip/brotli/deflate/zstd encoders)
/// - In-memory cached file streams ([`crate::mem_cache::stream::MemCacheFileStream`])
pub type Body = BoxBody<Bytes, io::Error>;

/// Creates an empty body (zero bytes).
///
/// Replaces `hyper::Body::empty()`.
#[inline]
pub fn empty() -> Body {
    Empty::new().map_err(|never| match never {}).boxed()
}

/// Creates a full body from in-memory bytes.
///
/// Replaces `hyper::Body::from(x)` for byte buffers and strings.
/// Accepts anything that converts into [`Bytes`]: `String`, `Vec<u8>`, `&'static str`,
/// `&'static [u8]`, or `Bytes` directly.
#[inline]
pub fn full(bytes: impl Into<Bytes>) -> Body {
    Full::new(bytes.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Creates a streaming body from an async byte stream.
///
/// Replaces `hyper::Body::wrap_stream(s)`.
///
/// The stream must yield `Result<Bytes, io::Error>`. Each successful item is
/// wrapped as an HTTP data [`Frame`] before being fed into the body.
pub fn stream<S>(s: S) -> Body
where
    S: futures_util::TryStream<Ok = Bytes, Error = io::Error> + Send + Sync + 'static,
{
    use futures_util::TryStreamExt as _;
    StreamBody::new(s.map_ok(Frame::data)).boxed()
}
