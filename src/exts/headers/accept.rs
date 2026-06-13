// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

use headers::{Error, Header};
use hyper::header::{ACCEPT, HeaderName, HeaderValue};

use super::QualityValue;

/// `Accept` header, defined in
/// [RFC7231](https://tools.ietf.org/html/rfc7231#section-5.3.2)
///
/// The `Accept` header field can be used by user agents to
/// specify their preferences regarding response media types.
///
/// # ABNF
///
/// ```text
/// Accept = #( media-range [ accept-params ] )
/// media-range = ( "*/*"
///               / ( type "/" "*" )
///               / ( type "/" subtype )
///               ) *( OWS ";" OWS parameter )
/// ```
///
/// # Example Values
///
/// * `text/html`
/// * `text/markdown, text/html;q=0.9`
/// * `text/*;q=0.8, application/json`
///
#[derive(Clone, Debug)]
pub(crate) struct Accept(QualityValue);

impl Header for Accept {
    fn name() -> &'static HeaderName {
        &ACCEPT
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

impl Accept {
    /// Check if a specific media type is accepted
    pub(crate) fn accepts(&self, media_type: &str) -> bool {
        self.0
            .iter()
            .any(|value| value.eq_ignore_ascii_case(media_type))
    }

    /// Returns true if text/markdown is explicitly accepted
    pub(crate) fn accepts_markdown(&self) -> bool {
        self.accepts("text/markdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_markdown_explicit() {
        let val = HeaderValue::from_static("text/markdown");
        let accept = Accept(val.into());
        assert!(accept.accepts_markdown());
    }

    #[test]
    fn accepts_markdown_with_quality() {
        let val = HeaderValue::from_static("text/html, text/markdown;q=0.9");
        let accept = Accept(val.into());
        assert!(accept.accepts_markdown());
    }

    #[test]
    fn does_not_accept_markdown_wildcard() {
        let val = HeaderValue::from_static("text/*");
        let accept = Accept(val.into());
        assert!(!accept.accepts_markdown());
    }

    #[test]
    fn does_not_accept_markdown_any() {
        let val = HeaderValue::from_static("*/*");
        let accept = Accept(val.into());
        assert!(!accept.accepts_markdown());
    }

    #[test]
    fn does_not_accept_markdown_html() {
        let val = HeaderValue::from_static("text/html, application/json");
        let accept = Accept(val.into());
        assert!(!accept.accepts_markdown());
    }
}
