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

    /// Returns the weight (`0` to `1000`) of `media_type` when the client lists
    /// it by name. Wildcard ranges don't count. `None` when it isn't listed.
    pub(crate) fn listed_quality(&self, media_type: &str) -> Option<u16> {
        self.0
            .iter_weighted()
            .find(|(range, _)| range.eq_ignore_ascii_case(media_type))
            .map(|(_, quality)| quality)
    }

    /// Returns the weight (`0` to `1000`) of the most specific media range
    /// that matches `media_type`, per RFC 9110, section 12.5.1. `None` when
    /// no range matches.
    pub(crate) fn quality(&self, media_type: &str) -> Option<u16> {
        let (main_type, _) = media_type.split_once('/')?;
        let mut best: Option<(u8, u16)> = None;
        for (range, quality) in self.0.iter_weighted() {
            let specificity = match range.split_once('/') {
                _ if range.eq_ignore_ascii_case(media_type) => 2,
                Some((range_type, "*")) if range_type.eq_ignore_ascii_case(main_type) => 1,
                Some(("*", "*")) => 0,
                _ => continue,
            };
            if best.is_none_or(|(best_specificity, _)| specificity > best_specificity) {
                best = Some((specificity, quality));
            }
        }
        best.map(|(_, quality)| quality)
    }

    /// Returns the first listed media type among `candidates`, honoring the
    /// media types q-value order. `None` when none of the candidates is listed.
    pub(crate) fn preferred_media_type<'a>(&self, candidates: &[&'a str]) -> Option<&'a str> {
        self.0.iter().find_map(|value| {
            candidates
                .iter()
                .find(|candidate| value.eq_ignore_ascii_case(candidate))
                .copied()
        })
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

    #[test]
    fn listed_quality_ignores_wildcards() {
        let val = HeaderValue::from_static("image/webp;q=0.5, image/*, */*");
        let accept = Accept(val.into());
        assert_eq!(accept.listed_quality("image/webp"), Some(500));
        assert_eq!(accept.listed_quality("IMAGE/WEBP"), Some(500));
        assert_eq!(accept.listed_quality("image/avif"), None);
    }

    #[test]
    fn listed_quality_keeps_not_acceptable() {
        let val = HeaderValue::from_static("image/avif;q=0, image/webp");
        let accept = Accept(val.into());
        assert_eq!(accept.listed_quality("image/avif"), Some(0));
        assert_eq!(accept.listed_quality("image/webp"), Some(1000));
    }

    #[test]
    fn quality_uses_most_specific_range() {
        let val = HeaderValue::from_static("*/*;q=0.1, image/*;q=0.5, image/png;q=0.8");
        let accept = Accept(val.into());
        assert_eq!(accept.quality("image/png"), Some(800));
        assert_eq!(accept.quality("image/jpeg"), Some(500));
        assert_eq!(accept.quality("text/html"), Some(100));

        // Order doesn't matter, only specificity
        let val = HeaderValue::from_static("image/png;q=0.8, image/*;q=0.5, */*;q=0.1");
        let accept = Accept(val.into());
        assert_eq!(accept.quality("image/png"), Some(800));
        assert_eq!(accept.quality("image/gif"), Some(500));
    }

    #[test]
    fn quality_honors_not_acceptable_over_wildcard() {
        let val = HeaderValue::from_static("image/png;q=0, */*");
        let accept = Accept(val.into());
        assert_eq!(accept.quality("image/png"), Some(0));
        assert_eq!(accept.quality("image/jpeg"), Some(1000));
    }

    #[test]
    fn quality_none_without_matching_range() {
        let val = HeaderValue::from_static("text/html, image/webp");
        let accept = Accept(val.into());
        assert_eq!(accept.quality("image/png"), None);
        assert_eq!(accept.quality("invalid"), None);
    }

    #[test]
    fn preferred_media_type_json_first() {
        let val = HeaderValue::from_static("application/json, text/html");
        let accept = Accept(val.into());
        assert_eq!(
            accept.preferred_media_type(&["application/json", "text/html"]),
            Some("application/json")
        );
    }

    #[test]
    fn preferred_media_type_html_first() {
        let val = HeaderValue::from_static("text/html, application/json");
        let accept = Accept(val.into());
        assert_eq!(
            accept.preferred_media_type(&["application/json", "text/html"]),
            Some("text/html")
        );
    }

    #[test]
    fn preferred_media_type_respects_q_values() {
        // JSON preferred via a higher q-value despite appearing second
        let val = HeaderValue::from_static("text/html;q=0.8, application/json;q=0.9");
        let accept = Accept(val.into());
        assert_eq!(
            accept.preferred_media_type(&["application/json", "text/html"]),
            Some("application/json")
        );
    }

    #[test]
    fn preferred_media_type_none_when_absent() {
        let val = HeaderValue::from_static("text/plain, */*");
        let accept = Accept(val.into());
        assert_eq!(
            accept.preferred_media_type(&["application/json", "text/html"]),
            None
        );
    }

    #[test]
    fn preferred_media_type_case_insensitive() {
        let val = HeaderValue::from_static("Application/JSON");
        let accept = Accept(val.into());
        assert_eq!(
            accept.preferred_media_type(&["application/json"]),
            Some("application/json")
        );
    }
}
