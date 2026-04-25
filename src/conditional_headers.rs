// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that provides HTTP header conditionals.
//!

use crate::body::Body;
use headers::{
    HeaderMap, HeaderMapExt, HeaderValue, IfModifiedSince, IfRange, IfUnmodifiedSince,
    LastModified, Range,
};
use hyper::{Response, StatusCode};

#[derive(Debug)]
pub(crate) struct ConditionalHeaders {
    pub(crate) if_modified_since: Option<IfModifiedSince>,
    pub(crate) if_unmodified_since: Option<IfUnmodifiedSince>,
    pub(crate) if_range: Option<IfRange>,
    pub(crate) range: Option<Range>,
}

impl ConditionalHeaders {
    pub(crate) fn new(headers: &HeaderMap<HeaderValue>) -> Self {
        let if_modified_since = headers.typed_get::<IfModifiedSince>();
        let if_unmodified_since = headers.typed_get::<IfUnmodifiedSince>();
        let if_range = headers.typed_get::<IfRange>();
        let range = headers.typed_get::<Range>();

        Self {
            if_modified_since,
            if_unmodified_since,
            if_range,
            range,
        }
    }
}

impl ConditionalHeaders {
    pub(crate) fn check(self, last_modified: Option<LastModified>) -> ConditionalBody {
        if let Some(since) = self.if_unmodified_since {
            let precondition = last_modified
                .map(|time| since.precondition_passes(time.into()))
                .unwrap_or(false);

            tracing::trace!(
                "if-unmodified-since? {:?} vs {:?} = {}",
                since,
                last_modified,
                precondition
            );
            if !precondition {
                let mut res = Response::new(crate::body::empty());
                *res.status_mut() = StatusCode::PRECONDITION_FAILED;
                return ConditionalBody::NoBody(res);
            }
        }

        if let Some(since) = self.if_modified_since {
            tracing::trace!(
                "if-modified-since? header = {:?}, file = {:?}",
                since,
                last_modified
            );
            let unmodified = last_modified
                .map(|time| !since.is_modified(time.into()))
                // no last_modified means its always modified
                .unwrap_or(false);
            if unmodified {
                let mut res = Response::new(crate::body::empty());
                *res.status_mut() = StatusCode::NOT_MODIFIED;
                return ConditionalBody::NoBody(res);
            }
        }

        if let Some(if_range) = self.if_range {
            tracing::trace!("if-range? {:?} vs {:?}", if_range, last_modified);

            let can_range = !if_range.is_modified(None, last_modified.as_ref());
            if !can_range {
                return ConditionalBody::WithBody(None);
            }
        }

        ConditionalBody::WithBody(self.range)
    }
}

pub(crate) enum ConditionalBody {
    NoBody(Response<Body>),
    WithBody(Option<Range>),
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use headers::{
        HeaderMap, HeaderMapExt, HeaderValue, IfModifiedSince, IfUnmodifiedSince, LastModified,
    };
    use hyper::StatusCode;

    use super::{ConditionalBody, ConditionalHeaders};

    fn make_last_modified(secs_ago: u64) -> LastModified {
        let time = SystemTime::now() - Duration::from_secs(secs_ago);
        LastModified::from(time)
    }

    #[test]
    fn new_empty_headers_all_none() {
        let headers = HeaderMap::new();
        let cond = ConditionalHeaders::new(&headers);
        assert!(cond.if_modified_since.is_none());
        assert!(cond.if_unmodified_since.is_none());
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
    fn check_no_conditionals_returns_with_body() {
        let headers = HeaderMap::new();
        let cond = ConditionalHeaders::new(&headers);
        match cond.check(None) {
            ConditionalBody::WithBody(range) => assert!(range.is_none()),
            ConditionalBody::NoBody(_) => panic!("expected WithBody"),
        }
    }

    #[test]
    fn check_if_modified_since_not_modified_returns_304() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        // Resource was last modified 1 hour ago.
        // Client: "give me only if modified since 30 min ago"
        let client_since = SystemTime::now() - Duration::from_secs(1800);
        headers.typed_insert(IfModifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        // Last modified 1 hour ago — not modified since client's timestamp
        let last_mod = make_last_modified(3600);
        match cond.check(Some(last_mod)) {
            ConditionalBody::NoBody(resp) => assert_eq!(resp.status(), StatusCode::NOT_MODIFIED),
            ConditionalBody::WithBody(_) => panic!("expected NoBody 304"),
        }
    }

    #[test]
    fn check_if_modified_since_modified_returns_with_body() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        // Client cached version from 2 hours ago.
        // Resource updated 30 min ago
        let client_since = SystemTime::now() - Duration::from_secs(7200);
        headers.typed_insert(IfModifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        let last_mod = make_last_modified(1800);
        match cond.check(Some(last_mod)) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("expected WithBody"),
        }
    }

    #[test]
    fn check_if_unmodified_since_precondition_fails_returns_412() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        // Client: "only respond if NOT modified since 2 hours ago"
        // Resource was modified 30 min ago — precondition fails
        let client_since = SystemTime::now() - Duration::from_secs(7200);
        headers.typed_insert(IfUnmodifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        let last_mod = make_last_modified(1800);
        match cond.check(Some(last_mod)) {
            ConditionalBody::NoBody(resp) => {
                assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED)
            }
            ConditionalBody::WithBody(_) => panic!("expected NoBody 412"),
        }
    }

    #[test]
    fn check_if_unmodified_since_precondition_passes_returns_with_body() {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        // Client: "only respond if NOT modified since 30 min ago"
        // Resource was last modified 2 hours ago — precondition passes
        let client_since = SystemTime::now() - Duration::from_secs(1800);
        headers.typed_insert(IfUnmodifiedSince::from(client_since));
        let cond = ConditionalHeaders::new(&headers);
        let last_mod = make_last_modified(7200);
        match cond.check(Some(last_mod)) {
            ConditionalBody::WithBody(_) => {}
            ConditionalBody::NoBody(_) => panic!("expected WithBody"),
        }
    }
}
