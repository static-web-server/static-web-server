// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to handle HTTP `Range` headers and byte ranges.
//!

use headers::{Header, Range};
use std::ops::Bound;

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

    for (start, end) in range.satisfiable_ranges(max_len) {
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
    // RFC 9110 §14.1.2 and should return the entire file (200), but headers 0.4
    // `satisfiable_ranges(len)` filters them out. Inspect the original header on
    // this cold path so out-of-bounds first-byte ranges (e.g. bytes=5000-) remain
    // unsatisfiable instead of being mistaken for an oversized suffix range.
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
    for b in digits.bytes() {
        if !b.is_ascii_digit() {
            return false;
        }
        let digit = u64::from(b - b'0');
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
    fn enormous_suffix_range_returns_full() {
        assert_eq!(
            bytes_range(range("-18446744073709551616"), 1000).unwrap(),
            (0, 1000)
        );
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
        // bytes=-1000 of 1000-byte file → entire file (last 1000 bytes == full)
        assert_eq!(bytes_range(range("-1000"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn suffix_range_of_one_byte() {
        // bytes=-1 of 1000-byte file → final byte = (999, 1000)
        assert_eq!(bytes_range(range("-1"), 1000).unwrap(), (999, 1000));
    }

    #[test]
    fn zero_length_suffix_is_unsatisfiable() {
        // bytes=-0 is explicitly forbidden by RFC 9110; we return 416.
        assert!(bytes_range(range("-0"), 1000).is_err());
    }

    #[test]
    fn leading_zeros_in_suffix_are_parsed() {
        // bytes=-00200 == bytes=-200
        assert_eq!(bytes_range(range("-00200"), 1000).unwrap(), (800, 1000));
        // bytes=-02000 on 1000-byte file → oversized → full file.
        assert_eq!(bytes_range(range("-02000"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn full_file_range_returns_full() {
        // bytes=0-999 on 1000-byte file == full file as partial content.
        assert_eq!(bytes_range(range("0-999"), 1000).unwrap(), (0, 1000));
    }

    #[test]
    fn multi_range_first_satisfiable_wins() {
        // Both within bounds — first one wins.
        assert_eq!(bytes_range(range("0-9,50-59"), 100).unwrap(), (0, 10));
    }

    #[test]
    fn multi_range_with_oversized_suffix_picks_first_valid() {
        // First range is valid; oversized suffix never reached.
        assert_eq!(bytes_range(range("10-19,-99999"), 100).unwrap(), (10, 20));
    }

    #[test]
    fn range_at_u64_boundary_file_size() {
        // Open-ended range on a (notionally) u64::MAX-sized file shouldn't overflow.
        assert_eq!(bytes_range(range("0-"), u64::MAX).unwrap(), (0, u64::MAX));
    }

    // Property-based regression tests for `bytes_range`.
    //
    // These properties encode invariants that should hold for any
    // satisfiable response: the returned slice must always be in-bounds
    // and non-empty, and well-formed single-range requests must round-trip
    // through the function with the expected semantics.
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 512, ..ProptestConfig::default()
        })]

        /// Inclusive range `bytes=a-b` with `a <= b < max_len` MUST yield
        /// exactly `(a, b + 1)`.
        #[test]
        fn prop_inclusive_range_roundtrip(
            (max_len, a, b) in (1u64..=10_000)
                .prop_flat_map(|m| (Just(m), 0u64..m, 0u64..m))
                .prop_map(|(m, x, y)| (m, x.min(y), x.max(y))),
        ) {
            let req = format!("{a}-{b}");
            let (start, end) = bytes_range(range(&req), max_len).unwrap();
            prop_assert_eq!((start, end), (a, b + 1));
        }

        /// Open-ended range `bytes=a-` with `a < max_len` MUST yield `(a, max_len)`.
        #[test]
        fn prop_open_ended_range_extends_to_eof(
            (max_len, a) in (1u64..=10_000).prop_flat_map(|m| (Just(m), 0u64..m)),
        ) {
            let req = format!("{a}-");
            let (start, end) = bytes_range(range(&req), max_len).unwrap();
            prop_assert_eq!((start, end), (a, max_len));
        }

        /// Suffix range `bytes=-n` with `1 <= n <= max_len` MUST yield the last
        /// `n` bytes, i.e. `(max_len - n, max_len)`.
        #[test]
        fn prop_suffix_range_within_bounds(
            (max_len, n) in (1u64..=10_000).prop_flat_map(|m| (Just(m), 1u64..=m)),
        ) {
            let req = format!("-{n}");
            let (start, end) = bytes_range(range(&req), max_len).unwrap();
            prop_assert_eq!((start, end), (max_len - n, max_len));
        }

        /// Suffix range `bytes=-n` with `n > max_len > 0` MUST yield the full
        /// file `(0, max_len)` per RFC 9110 §14.1.2.
        #[test]
        fn prop_oversized_suffix_returns_full_file(
            (max_len, n) in (1u64..=10_000)
                .prop_flat_map(|m| (Just(m), (m + 1)..=u64::MAX)),
        ) {
            let req = format!("-{n}");
            let (start, end) = bytes_range(range(&req), max_len).unwrap();
            prop_assert_eq!((start, end), (0, max_len));
        }

        /// Every successful response MUST be a non-empty, in-bounds slice:
        /// `start < end <= max_len`.
        #[test]
        fn prop_successful_slice_is_in_bounds(
            max_len in 1u64..=10_000,
            spec in "(\\PC{0,32})",
        ) {
            // Try to parse arbitrary text as a Range; if it parses, the
            // result must be a valid slice.
            let mut map = HeaderMap::new();
            if let Ok(value) = format!("bytes={spec}").parse() {
                map.insert(http::header::RANGE, value);
                let parsed = map.typed_get::<Range>();
                if let Ok((start, end)) = bytes_range(parsed, max_len) {
                    prop_assert!(start < end, "empty slice: {start}-{end}");
                    prop_assert!(end <= max_len, "end > max_len: {end} > {max_len}");
                }
            }
        }

        /// A first-byte range starting at or past EOF MUST never succeed.
        #[test]
        fn prop_out_of_bounds_first_byte_is_unsatisfiable(
            max_len in 1u64..=10_000,
            offset in 0u64..u64::MAX / 2,
        ) {
            let start = max_len.saturating_add(offset);
            let req = format!("{start}-");
            prop_assert!(bytes_range(range(&req), max_len).is_err());
        }
    }
}
