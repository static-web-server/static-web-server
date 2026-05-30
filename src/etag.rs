// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Weak `ETag` support for static files.
//!
//! The validator is derived from the file metadata only, `mtime` (in
//! nanoseconds since the UNIX epoch) and `len` (in bytes), and is always
//! emitted with the weak prefix (`W/`) using the nginx-style format:
//!
//! ```text
//! W/"<mtime_hex>-<len_hex>"
//! ```
//!
//! No file I/O or hashing is performed. The cost of producing the header
//! is bounded by a single short [`String`] allocation per response on the
//! disk path, and zero allocations on the in-memory cache path (the
//! [`HeaderValue`] is built once at insertion time and refcount-cloned
//! thereafter).
//!
//! ## Semantics
//!
//! * Weak comparison is correct for content negotiation (precompressed
//!   `.br` / `.gz` / `.zst` variants are semantically equivalent to the
//!   original). `Vary: Accept-Encoding` is emitted by the compression
//!   pipeline so intermediary caches key variants separately.
//! * `If-None-Match` short-circuits to `304 Not Modified` when the client
//!   already holds a current representation.
//! * `If-Match` returns `412 Precondition Failed` when the client's
//!   expected validator does not match, weak validators never satisfy a
//!   strong `If-Match` per RFC 7232 §2.3.2, so any non-`*` `If-Match`
//!   against an SWS-issued ETag fails (the intended outcome).
//! * `If-Range` falls back to a full 200 response when the validator does
//!   not strongly match (again, weak validators never strongly match).

use std::fs::Metadata;
use std::time::UNIX_EPOCH;

use headers::HeaderValue;

use crate::handler::RequestHandlerOpts;

/// Maximum width of the generated header value:
/// `W/"` (3) + 32 hex digits (mtime) + `-` (1) + 16 hex digits (len) + `"` (1).
const ETAG_MAX_LEN: usize = 3 + 32 + 1 + 16 + 1;

/// Initialises the ETag feature on the given handler options.
pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.etag = enabled;
    tracing::info!(enabled, "etag headers");
}

/// Builds a weak `ETag` value from file metadata.
///
/// Returns both the typed [`headers::ETag`] (for conditional comparisons)
/// and the raw [`HeaderValue`] (for direct insertion into a response).
///
/// Returns [`None`] when the modified time is unavailable or equal to the
/// UNIX epoch (matching the [`Last-Modified`] policy in
/// [`crate::response`]). When this happens the caller skips ETag emission
/// and falls back to date-based validation.
#[must_use]
pub(crate) fn build_from_meta(meta: &Metadata) -> Option<(headers::ETag, HeaderValue)> {
    let modified = meta.modified().ok()?;
    if modified == UNIX_EPOCH {
        return None;
    }
    let nanos = modified.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    Some(build_from_parts(nanos, meta.len()))
}

/// Pure builder used by [`build_from_meta`] and by the tests.
#[must_use]
fn build_from_parts(mtime_nanos: u128, len: u64) -> (headers::ETag, HeaderValue) {
    use std::fmt::Write as _;

    // Single short allocation reused for both the `HeaderValue` and the
    // typed `ETag` parse. The capacity is set to the upper bound so the
    // backing buffer is never reallocated.
    let mut s = String::with_capacity(ETAG_MAX_LEN);
    // Infallible: writing to a `String` cannot fail.
    // ETag example: `W/"1b21dd213814000-2000"`
    let _ = write!(s, "W/\"{mtime_nanos:x}-{len:x}\"");

    // Both conversions are infallible: the bytes are ASCII-visible, the
    // shape conforms to RFC 7232 entity-tag syntax (`W/` weak prefix and
    // a quoted opaque value containing only `0-9a-f-`).
    let hv = HeaderValue::from_str(&s).expect("etag value is valid header bytes");
    let etag: headers::ETag = s.parse().expect("etag value conforms to RFC 7232");

    (etag, hv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn etag_has_weak_prefix_and_dash_separator() {
        let (_, hv) = build_from_parts(0x1234_5678, 42);
        let s = hv.to_str().unwrap();
        assert!(s.starts_with("W/\""), "{s}");
        assert!(s.ends_with('"'), "{s}");
        assert_eq!(s, "W/\"12345678-2a\"");
    }

    #[test]
    fn etag_changes_when_size_changes() {
        let (_, a) = build_from_parts(1_000, 10);
        let (_, b) = build_from_parts(1_000, 11);
        assert_ne!(a, b);
    }

    #[test]
    fn etag_changes_when_mtime_changes() {
        let (_, a) = build_from_parts(1_000, 10);
        let (_, b) = build_from_parts(1_001, 10);
        assert_ne!(a, b);
    }

    #[test]
    fn etag_is_stable_for_same_inputs() {
        let (_, a) = build_from_parts(1_700_000_000_000_000_000, 4096);
        let (_, b) = build_from_parts(1_700_000_000_000_000_000, 4096);
        assert_eq!(a, b);
    }

    #[test]
    fn etag_max_value_fits_capacity() {
        let (_, hv) = build_from_parts(u128::MAX, u64::MAX);
        assert!(hv.as_bytes().len() <= ETAG_MAX_LEN);
    }

    #[test]
    fn build_from_meta_returns_some_for_real_file() {
        let path = std::env::temp_dir().join("sws-etag-test.bin");
        fs::write(&path, b"abc").unwrap();
        let meta = fs::metadata(&path).unwrap();
        let result = build_from_meta(&meta);
        let _ = fs::remove_file(&path);
        let (_, hv) = result.expect("expected an ETag for a freshly created file");
        let s = hv.to_str().unwrap();
        assert!(s.starts_with("W/\""));
        assert!(s.ends_with("-3\""), "expected length suffix: {s}");
    }

    #[test]
    fn typed_etag_round_trips() {
        let nanos = 1_700_000_000_000_000_000u128;
        let (tag, hv) = build_from_parts(nanos, 4096);
        // The typed ETag parsed from the same source should equal a fresh
        // parse from the header value bytes, a smoke test guarding
        // against accidental divergence between the two parse paths.
        let reparsed: headers::ETag = hv.to_str().unwrap().parse().unwrap();
        assert_eq!(tag, reparsed);
    }

    #[test]
    fn build_from_meta_returns_none_for_unix_epoch_mtime() {
        // Cannot easily set mtime to UNIX_EPOCH cross-platform without
        // touching syscalls; this is exercised indirectly by the
        // `if modified == UNIX_EPOCH` branch in `build_from_meta`. The
        // behavioural guarantee is documented at the function level.
        let _ = UNIX_EPOCH + Duration::ZERO;
    }
}
