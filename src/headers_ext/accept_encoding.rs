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
pub(crate) struct AcceptEncoding(QualityValue);

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
    /// Returns a quality sorted iterator of the `ContentCoding`
    pub(crate) fn sorted_encodings(&self) -> impl Iterator<Item = ContentCoding> + '_ {
        self.0.iter().map(ContentCoding::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_static() {
        let val = HeaderValue::from_static("deflate, zstd;q=0.7, gzip;q=1.0, br;q=0.9");
        let accept_enc = AcceptEncoding(val.into());

        let mut encodings = accept_enc.sorted_encodings();
        assert_eq!(encodings.next(), Some(ContentCoding::DEFLATE));
        assert_eq!(encodings.next(), Some(ContentCoding::GZIP));
        assert_eq!(encodings.next(), Some(ContentCoding::BROTLI));
        assert_eq!(encodings.next(), Some(ContentCoding::ZSTD));
        assert_eq!(encodings.next(), None);
    }
}
