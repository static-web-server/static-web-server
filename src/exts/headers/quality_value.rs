// SPDX-License-Identifier: MIT OR Apache-2.0
// Original code by Parker Timmerman
// Original code sourced from https://github.com/hyperium/headers/pull/70

use bytes::BytesMut;
use headers::Error;
use hyper::header::HeaderValue;
use std::cmp::Ordering;

use super::ContentCoding;

/// A CSV list that respects the Quality Values syntax defined in
/// [RFC7321](https://tools.ietf.org/html/rfc7231#section-5.3.1)
///
/// Many of the request header fields for proactive negotiation use a
/// common parameter, named "q" (case-insensitive), to assign a relative
/// "weight" to the preference for that associated kind of content.  This
/// weight is referred to as a "quality value" (or "qvalue") because the
/// same parameter name is often used within server configurations to
/// assign a weight to the relative quality of the various
/// representations that can be selected for a resource.
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct QualityValue {
    value: HeaderValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct QualityMeta<'a> {
    pub data: &'a str,
    pub quality: u16,
}

impl<'a> TryFrom<&'a str> for QualityMeta<'a> {
    type Error = Error;

    fn try_from(val: &'a str) -> Result<Self, Error> {
        let data = val.split(';').next().ok_or(Error::invalid())?.trim();

        // Default quality value is 1
        let mut quality = 1000u16;
        // Only scan for q= after the first semicolon
        if let Some(rest) = val.split_once(';').map(|(_, r)| r) {
            for part in rest.split(';') {
                let part = part.trim_start();
                if let Some(value) = part.strip_prefix("q=")
                    && let Ok(parsed) = value.trim().parse::<f32>()
                {
                    quality = (parsed * 1000_f32) as u16;
                }
            }
        }

        Ok(QualityMeta { data, quality })
    }
}

impl QualityValue {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &str> + use<'_> {
        let mut items: Vec<_> = self
            .value
            .to_str()
            .ok()
            .into_iter()
            .flat_map(|value_str| value_str.split(','))
            .filter_map(|v| QualityMeta::try_from(v).ok())
            .collect();
        items.sort_unstable_by(|a, b| {
            let quality_cmp = b.quality.cmp(&a.quality);
            if quality_cmp == Ordering::Equal {
                let priority_a = ContentCoding::from(a.data).priority();
                let priority_b = ContentCoding::from(b.data).priority();
                priority_b.cmp(&priority_a)
            } else {
                quality_cmp
            }
        });
        items.into_iter().map(|pair| pair.data)
    }

    pub(crate) fn try_from_values<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let mut values = values.peekable();
        let first = match values.next() {
            Some(item) => item,
            None => return Ok(HeaderValue::from_static("").into()),
        };

        if values.peek().is_none() {
            // Only one item, can be returned directly
            return Ok(first.into());
        }

        let mut buf = BytesMut::from(first.as_bytes());
        for value in values {
            buf.extend_from_slice(b", ");
            buf.extend_from_slice(value.as_bytes());
        }

        Ok(HeaderValue::from_maybe_shared(buf.freeze())
            .map_err(|_| Error::invalid())?
            .into())
    }
}

impl<F: Into<f32>> TryFrom<(&str, F)> for QualityValue {
    type Error = Error;

    fn try_from(pair: (&str, F)) -> Result<Self, Error> {
        let (data, quality) = pair;
        let value = HeaderValue::try_from(format!("{data};q={}", quality.into()))
            .map_err(|_e| Error::invalid())?;
        Ok(Self::from(value))
    }
}

impl From<HeaderValue> for QualityValue {
    fn from(value: HeaderValue) -> Self {
        QualityValue { value }
    }
}

impl From<&HeaderValue> for QualityValue {
    fn from(value: &HeaderValue) -> Self {
        QualityValue {
            value: value.clone(),
        }
    }
}

impl From<&QualityValue> for HeaderValue {
    fn from(qual: &QualityValue) -> Self {
        qual.value.clone()
    }
}

impl From<QualityValue> for HeaderValue {
    fn from(qual: QualityValue) -> Self {
        qual.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn without_qualities() {
        let val = HeaderValue::from_static("zstd, br, gzip, deflate");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("zstd"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("deflate"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn without_qualities_wrong_order() {
        let val = HeaderValue::from_static("gzip, deflate, br, zstd");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("zstd"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("deflate"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn honor_client_preference_order() {
        let val = HeaderValue::from_static("gzip, br;q=0.5, zstd;q=0");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), Some("zstd"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn multiple_qualities() {
        let val = HeaderValue::from_static("gzip;q=1, br;q=0.8");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn multiple_qualities_wrong_order() {
        let val = HeaderValue::from_static("br;q=0.8, gzip;q=1.0");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn multiple_values() {
        let val = HeaderValue::from_static("deflate, gzip;q=1, br;q=0.8");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("deflate"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn multiple_values_whitespace() {
        let val = HeaderValue::from_static("deflate  ;q=0.9, br;   q=0.001 ,gzip  ;  q=0.8");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("deflate"));
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn multiple_values_wrong_order() {
        let val = HeaderValue::from_static("deflate, br;q=0.8, gzip;q=1, *;q=0.1");
        let qual = QualityValue::from(val);

        let mut values = qual.iter();
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("deflate"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), Some("*"));
        assert_eq!(values.next(), None);
    }

    // Property-based regression tests for the RFC 7231 q-value parser.
    //
    // Inputs to `QualityValue::iter` come straight from a client
    // `Accept-Encoding` (or similar) header, so the parser MUST be
    // total: no panic, no infinite loop, and ordering must respect the
    // documented monotonicity in `quality` then `ContentCoding::priority`.
    use proptest::prelude::*;

    /// Restrict to bytes the `HeaderValue` constructor accepts so we
    /// can exercise the parser path rather than rejecting input at the
    /// `HeaderValue` boundary.
    const HEADER_VALUE_REGEX: &str = "[A-Za-z0-9 ,;=\\.\\*/+\\-_]{0,256}";

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256, ..ProptestConfig::default()
        })]

        /// Iteration must be total: any header-value-safe input must
        /// drain `iter()` to completion without panicking.
        #[test]
        fn prop_iter_never_panics(s in HEADER_VALUE_REGEX) {
            let Ok(val) = HeaderValue::from_str(&s) else { return Ok(()); };
            let qual = QualityValue::from(val);
            // Drain the iterator fully.
            let _items: Vec<&str> = qual.iter().collect();
        }

        /// Iteration order respects q-value: a parsed q-value of 1.0
        /// must never appear after a parsed q-value of 0.5 for the
        /// same content.
        #[test]
        fn prop_iter_orders_q_values_descending(
            a in "[a-z]{1,8}",
            b in "[a-z]{1,8}",
            qa in 0u16..=1000,
            qb in 0u16..=1000,
        ) {
            prop_assume!(a != b);
            let s = format!("{a};q={:.3}, {b};q={:.3}", qa as f32 / 1000.0, qb as f32 / 1000.0);
            let Ok(val) = HeaderValue::from_str(&s) else { return Ok(()); };
            let qual = QualityValue::from(val);
            let items: Vec<&str> = qual.iter().collect();
            // Items are sorted descending by quality. Find positions
            // and verify their relative order matches the q-values.
            let pos_a = items.iter().position(|x| *x == a);
            let pos_b = items.iter().position(|x| *x == b);
            if let (Some(pa), Some(pb)) = (pos_a, pos_b) {
                match qa.cmp(&qb) {
                    std::cmp::Ordering::Greater => prop_assert!(pa < pb,
                        "expected `{a}` (q={qa}) before `{b}` (q={qb}) in {items:?}"),
                    std::cmp::Ordering::Less => prop_assert!(pa > pb,
                        "expected `{a}` (q={qa}) after `{b}` (q={qb}) in {items:?}"),
                    std::cmp::Ordering::Equal => {} // tie-broken by ContentCoding::priority
                }
            }
        }
    }
}
