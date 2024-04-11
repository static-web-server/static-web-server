// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that provides HTTP header conditionals.
//!

use headers::{
    HeaderMap, HeaderMapExt, HeaderValue, IfModifiedSince, IfRange, IfUnmodifiedSince,
    LastModified, Range,
};
use hyper::{Body, Response, StatusCode};

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
                let mut res = Response::new(Body::empty());
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
                let mut res = Response::new(Body::empty());
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
