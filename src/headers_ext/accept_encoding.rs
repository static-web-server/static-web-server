// SPDX-License-Identifier: MIT OR Apache-2.0
// Original code by Parker Timmerman
// Original code sourced from https://github.com/hyperium/headers/pull/70

use headers::{Error, Header};
use hyper::header::{HeaderName, HeaderValue, ACCEPT_ENCODING};

use super::{ContentCoding, QualityValue};

/// `Accept-Encoding` header, defined in
/// [RFC7231](https://tools.ietf.org/html/rfc7231#section-5.3.4)
///
/// The `Accept-Encoding` header field can be used by user agents to
/// indicate what response content-codings are acceptable in the response.
/// An "identity" token is used as a synonym for "no encoding" in
/// order to communicate when no encoding is preferred.
///
/// # ABNF
///
/// ```text
/// Accept-Encoding  = #( codings [ weight ] )
/// codings          = content-coding / "identity" / "*"
/// ```
///
/// # Example Values
///
/// * `gzip`
/// * `br;q=1.0, gzip;q=0.8`
///
#[derive(Clone, Debug)]
pub struct AcceptEncoding(pub QualityValue);

impl Header for AcceptEncoding {
    fn name() -> &'static HeaderName {
        &ACCEPT_ENCODING
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        QualityValue::try_from_values(values).map(Self)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once((&self.0).into()))
    }
}

impl AcceptEncoding {
    /// Convenience method to create an `Accept-Encoding: gzip` header
    #[inline]
    pub fn gzip() -> Self {
        Self(HeaderValue::from_static("gzip").into())
    }

    /// A convenience method to create an Accept-Encoding header from pairs of values and qualities
    ///
    /// # Example
    ///
    /// ```
    /// use static_web_server::headers_ext::AcceptEncoding;
    ///
    /// let pairs = vec![("gzip", 1.0), ("deflate", 0.8)];
    /// let header = AcceptEncoding::from_quality_pairs(&mut pairs.into_iter());
    /// ```
    pub fn from_quality_pairs<'i, I>(pairs: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = (&'i str, f32)>,
    {
        let values: Vec<HeaderValue> = pairs
            .map(|pair| {
                QualityValue::try_from(pair).map(|qual: QualityValue| HeaderValue::from(qual))
            })
            .collect::<Result<Vec<HeaderValue>, Error>>()?;
        let value = QualityValue::try_from_values(&mut values.iter())?;
        Ok(Self(value))
    }

    /// Returns the most preferred encoding that is specified by the header,
    /// if one is specified.
    ///
    /// Note: This peeks at the underlying iter, not modifying it.
    ///
    /// # Example
    ///
    /// ```
    /// use static_web_server::headers_ext::{AcceptEncoding, ContentCoding};
    ///
    /// let pairs = vec![("gzip", 1.0), ("deflate", 0.8)];
    /// let accept_enc = AcceptEncoding::from_quality_pairs(&mut pairs.into_iter()).unwrap();
    /// let mut encodings = accept_enc.sorted_encodings();
    ///
    /// assert_eq!(accept_enc.preferred_encoding(), Some(ContentCoding::GZIP));
    /// ```
    pub fn preferred_encoding(&self) -> Option<ContentCoding> {
        self.0.iter().next().map(ContentCoding::from)
    }

    /// Returns a quality sorted iterator of the `ContentCoding`
    ///
    /// # Example
    ///
    /// ```
    /// use headers::HeaderValue;
    /// use static_web_server::headers_ext::{AcceptEncoding, ContentCoding};
    ///
    /// let val = HeaderValue::from_static("deflate, gzip;q=1.0, br;q=0.8");
    /// let accept_enc = AcceptEncoding(val.into());
    /// let mut encodings = accept_enc.sorted_encodings();
    ///
    /// assert_eq!(encodings.next(), Some(ContentCoding::DEFLATE));
    /// assert_eq!(encodings.next(), Some(ContentCoding::GZIP));
    /// assert_eq!(encodings.next(), Some(ContentCoding::BROTLI));
    /// assert_eq!(encodings.next(), None);
    /// ```
    pub fn sorted_encodings(&self) -> impl Iterator<Item = ContentCoding> + '_ {
        self.0.iter().map(ContentCoding::from)
    }

    /// Returns a quality sorted iterator of values
    ///
    /// # Example
    ///
    /// ```
    /// use headers::HeaderValue;
    /// use static_web_server::headers_ext::{AcceptEncoding, ContentCoding};
    ///
    /// let val = HeaderValue::from_static("deflate, gzip;q=1.0, br;q=0.8");
    /// let accept_enc = AcceptEncoding(val.into());
    /// let mut encodings = accept_enc.sorted_values();
    ///
    /// assert_eq!(encodings.next(), Some("deflate"));
    /// assert_eq!(encodings.next(), Some("gzip"));
    /// assert_eq!(encodings.next(), Some("br"));
    /// assert_eq!(encodings.next(), None);
    /// ```
    pub fn sorted_values(&self) -> impl Iterator<Item = &str> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_static() {
        let val = HeaderValue::from_static("deflate, gzip;q=1.0, br;q=0.9");
        let accept_enc = AcceptEncoding(val.into());

        assert_eq!(
            accept_enc.preferred_encoding(),
            Some(ContentCoding::DEFLATE)
        );

        let mut encodings = accept_enc.sorted_encodings();
        assert_eq!(encodings.next(), Some(ContentCoding::DEFLATE));
        assert_eq!(encodings.next(), Some(ContentCoding::GZIP));
        assert_eq!(encodings.next(), Some(ContentCoding::BROTLI));
        assert_eq!(encodings.next(), None);
    }

    #[test]
    fn from_pairs() {
        let pairs = vec![("gzip", 1.0), ("br", 0.9)];
        let accept_enc = AcceptEncoding::from_quality_pairs(&mut pairs.into_iter()).unwrap();

        assert_eq!(accept_enc.preferred_encoding(), Some(ContentCoding::GZIP));

        let mut encodings = accept_enc.sorted_encodings();
        assert_eq!(encodings.next(), Some(ContentCoding::GZIP));
        assert_eq!(encodings.next(), Some(ContentCoding::BROTLI));
        assert_eq!(encodings.next(), None);
    }
}
