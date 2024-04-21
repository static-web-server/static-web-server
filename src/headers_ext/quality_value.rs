// SPDX-License-Identifier: MIT OR Apache-2.0
// Original code by Parker Timmerman
// Original code sourced from https://github.com/hyperium/headers/pull/70

use bytes::BytesMut;
use headers::Error;
use hyper::header::HeaderValue;
use std::cmp::Ordering;

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct QualityMeta<'a> {
    pub data: &'a str,
    pub quality: u16,
}

impl Ord for QualityMeta<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.quality.cmp(&self.quality)
    }
}

impl PartialOrd for QualityMeta<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> TryFrom<&'a str> for QualityMeta<'a> {
    type Error = Error;

    fn try_from(val: &'a str) -> Result<Self, Error> {
        let parts: Vec<_> = val.split(';').collect();
        let data = parts.first().ok_or(Error::invalid())?.trim();

        // Default quality value is 1
        let mut quality = 1000u16;
        for part in parts {
            let part = part.trim_start();
            if let Some(value) = part.strip_prefix("q=") {
                let parsed: f32 = match value.trim().parse() {
                    Ok(parsed) => parsed,
                    Err(_) => continue,
                };
                quality = (parsed * 1000_f32) as u16;
            }
        }

        Ok(QualityMeta { data, quality })
    }
}

impl QualityValue {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &str> {
        let mut items: Vec<_> = self
            .value
            .to_str()
            .ok()
            .into_iter()
            .flat_map(|value_str| value_str.split(','))
            .filter_map(|v| QualityMeta::try_from(v).ok())
            .collect();
        items.sort();
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
            buf.extend_from_slice(", ".as_bytes());
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
        assert_eq!(values.next(), Some("deflate"));
        assert_eq!(values.next(), Some("gzip"));
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
        assert_eq!(values.next(), Some("deflate"));
        assert_eq!(values.next(), Some("gzip"));
        assert_eq!(values.next(), Some("br"));
        assert_eq!(values.next(), Some("*"));
        assert_eq!(values.next(), None);
    }
}
