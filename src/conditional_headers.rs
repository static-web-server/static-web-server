// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that provides HTTP header conditionals.
//!

use crate::body::Body;
use headers::{
    ETag, HeaderMap, HeaderMapExt, HeaderValue, IfMatch, IfModifiedSince, IfNoneMatch, IfRange,
    IfUnmodifiedSince, LastModified, Range,
};
use hyper::header::ETAG;
use hyper::{Response, StatusCode};

#[derive(Debug)]
pub(crate) struct ConditionalHeaders {
    pub(crate) if_modified_since: Option<IfModifiedSince>,
    pub(crate) if_unmodified_since: Option<IfUnmodifiedSince>,
    pub(crate) if_match: Option<IfMatch>,
    pub(crate) if_none_match: Option<IfNoneMatch>,
    pub(crate) if_range: Option<IfRange>,
    pub(crate) range: Option<Range>,
}

impl ConditionalHeaders {
    pub(crate) fn new(headers: &HeaderMap<HeaderValue>) -> Self {
        let if_modified_since = headers.typed_get::<IfModifiedSince>();
        let if_unmodified_since = headers.typed_get::<IfUnmodifiedSince>();
        let if_match = headers.typed_get::<IfMatch>();
        let if_none_match = headers.typed_get::<IfNoneMatch>();
        let if_range = headers.typed_get::<IfRange>();
        let range = headers.typed_get::<Range>();

        Self {
            if_modified_since,
            if_unmodified_since,
            if_match,
            if_none_match,
            if_range,
            range,
        }
    }
}

/// Validator inputs passed to [`ConditionalHeaders::check`].
///
/// Both fields are optional: callers pass `None` when a validator is not
/// available (e.g. the file's `mtime` is the UNIX epoch, or the ETag
/// feature is disabled). The check then degrades gracefully — `If-Match`
/// / `If-None-Match` are skipped when no `ETag` is provided, and
/// `If-Modified-Since` / `If-Unmodified-Since` when no `LastModified` is
/// provided.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Validators<'a> {
    pub(crate) last_modified: Option<LastModified>,
    pub(crate) etag: Option<&'a ETag>,
    pub(crate) etag_value: Option<&'a HeaderValue>,
}

impl ConditionalHeaders {
    /// Evaluates conditional request headers against the resource's
    /// validators following the precedence rules of RFC 7232 §6:
    ///
    /// 1. `If-Match`            -> `412` on mismatch.
    /// 2. `If-Unmodified-Since` -> `412` on mismatch (ignored when `If-Match` is present).
    /// 3. `If-None-Match`       -> `304` on match.
    /// 4. `If-Modified-Since`   -> `304` on match (ignored when `If-None-Match` is present).
    /// 5. `If-Range`            -> drop `Range` on validator mismatch.
    pub(crate) fn check(self, v: Validators<'_>) -> ConditionalBody {
        // 1. If-Match takes precedence over If-Unmodified-Since.
        if let Some(if_match) = self.if_match.as_ref() {
            let passes = match v.etag {
                Some(etag) => if_match.precondition_passes(etag),
                // No ETag available: only `If-Match: *` can succeed,
                // since we still have a representation.
                None => if_match.is_any(),
            };

            tracing::trace!("if-match? {:?} vs {:?} = {}", if_match, v.etag, passes);
            if !passes {
                return ConditionalBody::NoBody(precondition_failed());
            }
        } else if let Some(since) = self.if_unmodified_since {
            let precondition = v
                .last_modified
                .map(|time| since.precondition_passes(time.into()))
                .unwrap_or(false);

            tracing::trace!(
                "if-unmodified-since? {:?} vs {:?} = {}",
                since,
                v.last_modified,
                precondition
            );
            if !precondition {
                return ConditionalBody::NoBody(precondition_failed());
            }
        }

        // 2. If-None-Match takes precedence over If-Modified-Since.
        if let Some(if_none_match) = self.if_none_match.as_ref() {
            if let Some(etag) = v.etag
                && !if_none_match.precondition_passes(etag)
            {
                tracing::trace!("if-none-match matched {:?}; returning 304", etag);
                return ConditionalBody::NoBody(not_modified(v));
            }
            // If no ETag was produced, fall through to date-based logic
            // (or no logic), emitting 304 for an absent validator would
            // be unsafe because the client may hold stale content.
        } else if let Some(since) = self.if_modified_since {
            tracing::trace!(
                "if-modified-since? header = {:?}, file = {:?}",
                since,
                v.last_modified
            );
            let unmodified = v
                .last_modified
                .map(|time| !since.is_modified(time.into()))
                .unwrap_or(false);
            if unmodified {
                return ConditionalBody::NoBody(not_modified(v));
            }
        }

        if let Some(if_range) = self.if_range {
            tracing::trace!(
                "if-range? {:?} vs etag={:?}, last_modified={:?}",
                if_range,
                v.etag,
                v.last_modified
            );

            let can_range = !if_range.is_modified(v.etag, v.last_modified.as_ref());
            if !can_range {
                return ConditionalBody::WithBody(None);
            }
        }

        ConditionalBody::WithBody(self.range)
    }
}

/// Builds the `412 Precondition Failed` response shared by the
/// `If-Match` and `If-Unmodified-Since` branches.
#[cold]
fn precondition_failed() -> Response<Body> {
    let mut res = Response::new(crate::body::empty());
    *res.status_mut() = StatusCode::PRECONDITION_FAILED;
    res
}

/// Builds a `304 Not Modified` response, echoing the validators that
/// the client may already hold (RFC 7232 §4.1).
fn not_modified(v: Validators<'_>) -> Response<Body> {
    let mut res = Response::new(crate::body::empty());
    *res.status_mut() = StatusCode::NOT_MODIFIED;
    if let Some(last_modified) = v.last_modified {
        res.headers_mut().typed_insert(last_modified);
    }
    if let Some(hv) = v.etag_value {
        res.headers_mut().insert(ETAG, hv.clone());
    }
    res
}

pub(crate) enum ConditionalBody {
    NoBody(Response<Body>),
    WithBody(Option<Range>),
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use headers::{
        ETag, HeaderMap, HeaderMapExt, HeaderValue, IfMatch, IfModifiedSince, IfNoneMatch,
        IfUnmodifiedSince, LastModified,
    };
    use hyper::StatusCode;

    use super::{ConditionalBody, ConditionalHeaders, Validators};

    fn make_last_modified(secs_ago: u64) -> LastModified {
        let time = SystemTime::now() - Duration::from_secs(secs_ago);
        LastModified::from(time)
    }

    fn weak_etag() -> (ETag, HeaderValue) {
        let s = "W/\"deadbeef-2a\"";
        (s.parse().unwrap(), HeaderValue::from_static(s))
    }

    fn other_etag() -> ETag {
        "W/\"cafebabe-99\"".parse().unwrap()
    }

    fn validator_from_last_mod(last_modified: Option<LastModified>) -> Validators<'static> {
        Validators {
            last_modified,
            etag: None,
            etag_value: None,
        }
    }

    #[test]
    fn new_empty_headers_all_none() {
        let headers = HeaderMap::new();
        let cond = ConditionalHeaders::new(&headers);
        assert!(cond.if_modified_since.is_none());
        assert!(cond.if_unmodified_since.is_none());
        assert!(cond.if_match.is_none());
        assert!(cond.if_none_match.is_none());
        assert!(cond.if_range.is_none());
        assert!(cond.range.is_none());
    }

    #[test]
    fn new_parses_if_modified_since() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfModifiedSince::from(
            std::time::SystemTime::now() - Duration::from_secs(3600),
        ));
        let cond = ConditionalHeaders::new(&headers);
        assert!(cond.if_modified_since.is_some());
    }

    #[test]
    fn new_parses_if_unmodified_since() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfUnmodifiedSince::from(std::time::SystemTime::now()));
        let cond = ConditionalHeaders::new(&headers);
        assert!(cond.if_unmodified_since.is_some());
    }

    #[test]
    fn new_parses_if_match_and_if_none_match() {
        let (etag, _) = weak_etag();
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfNoneMatch::from(etag.clone()));
        headers.typed_insert(IfMatch::any());
        let cond = ConditionalHeaders::new(&headers);
        assert!(cond.if_none_match.is_some());
        assert!(cond.if_match.is_some());
    }

    #[test]
    fn check_no_conditionals_returns_with_body() {
        let headers = HeaderMap::new();
        let cond = ConditionalHeaders::new(&headers);
        match cond.check(Validators::default()) {
            ConditionalBody::WithBody(range) => assert!(range.is_none()),
            ConditionalBody::NoBody(_) => panic!("expected WithBody"),
        }
    }

    #[test]
    fn check_if_modified_since_not_modified_returns_304() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        let client_since = SystemTime::now() - Duration::from_secs(1800);
        headers.typed_insert(IfModifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        let last_mod = make_last_modified(3600);
        match cond.check(validator_from_last_mod(Some(last_mod))) {
            ConditionalBody::NoBody(resp) => assert_eq!(resp.status(), StatusCode::NOT_MODIFIED),
            ConditionalBody::WithBody(_) => panic!("expected NoBody 304"),
        }
    }

    #[test]
    fn check_if_modified_since_modified_returns_with_body() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        let client_since = SystemTime::now() - Duration::from_secs(7200);
        headers.typed_insert(IfModifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        let last_mod = make_last_modified(1800);
        match cond.check(validator_from_last_mod(Some(last_mod))) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("expected WithBody"),
        }
    }

    #[test]
    fn check_if_unmodified_since_precondition_fails_returns_412() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        let client_since = SystemTime::now() - Duration::from_secs(7200);
        headers.typed_insert(IfUnmodifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        let last_mod = make_last_modified(1800);
        match cond.check(validator_from_last_mod(Some(last_mod))) {
            ConditionalBody::NoBody(resp) => {
                assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED)
            }
            ConditionalBody::WithBody(_) => panic!("expected NoBody 412"),
        }
    }

    #[test]
    fn check_if_unmodified_since_precondition_passes_returns_with_body() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        let client_since = SystemTime::now() - Duration::from_secs(1800);
        headers.typed_insert(IfUnmodifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        let last_mod = make_last_modified(7200);
        match cond.check(validator_from_last_mod(Some(last_mod))) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("expected WithBody"),
        }
    }

    // ETag-based conditionals

    #[test]
    fn check_if_none_match_matches_returns_304_with_etag_and_last_modified() {
        let (etag, etag_value) = weak_etag();
        let last_mod = make_last_modified(3600);
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfNoneMatch::from(etag.clone()));
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: Some(last_mod),
            etag: Some(&etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::NoBody(resp) => {
                assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
                // RFC 7232 §4.1: 304 must echo validators
                assert_eq!(resp.headers().get("etag").unwrap(), &etag_value);
                assert!(resp.headers().contains_key("last-modified"));
            }
            ConditionalBody::WithBody(_) => panic!("expected 304"),
        }
    }

    #[test]
    fn check_if_none_match_star_returns_304() {
        let (etag, etag_value) = weak_etag();
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfNoneMatch::any());
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: None,
            etag: Some(&etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::NoBody(resp) => assert_eq!(resp.status(), StatusCode::NOT_MODIFIED),
            ConditionalBody::WithBody(_) => panic!("expected 304"),
        }
    }

    #[test]
    fn check_if_none_match_mismatch_returns_with_body() {
        let (_, etag_value) = weak_etag();
        let cur_etag = "W/\"newer-1\"".parse::<ETag>().unwrap();
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfNoneMatch::from(other_etag()));
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: None,
            etag: Some(&cur_etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("expected 200"),
        }
    }

    #[test]
    fn check_if_none_match_takes_precedence_over_if_modified_since() {
        // A stale If-Modified-Since (would yield 304) is ignored when
        // If-None-Match mismatches (must yield 200).
        let (_, etag_value) = weak_etag();
        let cur_etag = "W/\"newer-1\"".parse::<ETag>().unwrap();
        let last_mod = make_last_modified(7200);
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfNoneMatch::from(other_etag()));
        headers.typed_insert(IfModifiedSince::from(
            SystemTime::now() - Duration::from_secs(1800),
        ));
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: Some(last_mod),
            etag: Some(&cur_etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("expected 200 (If-None-Match must override)"),
        }
    }

    #[test]
    fn check_if_match_with_weak_etag_fails_returns_412() {
        // RFC 7232: If-Match uses strong comparison; a weak validator
        // can never satisfy a non-`*` If-Match. SWS issues weak ETags
        // exclusively, so this is the expected outcome.
        let (etag, etag_value) = weak_etag();
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfMatch::from(etag.clone()));
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: None,
            etag: Some(&etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::NoBody(resp) => {
                assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED)
            }
            ConditionalBody::WithBody(_) => panic!("expected 412"),
        }
    }

    #[test]
    fn check_if_match_star_passes() {
        let (etag, etag_value) = weak_etag();
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfMatch::any());
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: None,
            etag: Some(&etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("If-Match: * must pass"),
        }
    }

    #[test]
    fn check_if_match_star_passes_even_without_etag() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfMatch::any());
        let cond = ConditionalHeaders::new(&headers);
        match cond.check(Validators::default()) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("If-Match: * must pass without an ETag"),
        }
    }

    #[test]
    fn check_if_match_without_etag_fails_returns_412() {
        let (etag, _) = weak_etag();
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfMatch::from(etag));
        let cond = ConditionalHeaders::new(&headers);
        // No ETag available: non-`*` If-Match must fail.
        match cond.check(Validators::default()) {
            ConditionalBody::NoBody(resp) => {
                assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED)
            }
            ConditionalBody::WithBody(_) => panic!("expected 412"),
        }
    }

    #[test]
    fn check_if_match_takes_precedence_over_if_unmodified_since() {
        // A passing If-Unmodified-Since is ignored when If-Match fails.
        let (etag, etag_value) = weak_etag();
        let last_mod = make_last_modified(7200);
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfMatch::from(etag.clone()));
        headers.typed_insert(IfUnmodifiedSince::from(
            SystemTime::now() - Duration::from_secs(1800),
        ));
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: Some(last_mod),
            etag: Some(&etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::NoBody(resp) => {
                assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED)
            }
            ConditionalBody::WithBody(_) => panic!("expected If-Match to override and 412"),
        }
    }

    #[test]
    fn not_modified_response_carries_etag_when_provided() {
        let (etag, etag_value) = weak_etag();
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.typed_insert(IfNoneMatch::any());
        let cond = ConditionalHeaders::new(&headers);
        let v = Validators {
            last_modified: None,
            etag: Some(&etag),
            etag_value: Some(&etag_value),
        };
        match cond.check(v) {
            ConditionalBody::NoBody(resp) => {
                assert_eq!(resp.headers().get("etag"), Some(&etag_value));
            }
            ConditionalBody::WithBody(_) => panic!("expected 304"),
        }
    }
}
