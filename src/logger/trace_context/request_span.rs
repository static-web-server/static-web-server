// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The span that carries the trace context of a request: its identity, which the JSON format
//! looks for, and its creation from the `traceparent` request header.

use hyper::header::HeaderMap;
use tracing::Span;

use super::traceparent::parse_traceparent;

/// Name of the span that carries the trace context of a request.
pub(crate) const NAME: &str = "trace_context";

/// Target of the span that carries the trace context of a request, and of the lines of the
/// trace context module. Kept from the former module path for log filters.
pub(super) const TARGET: &str = "static_web_server::trace_context";

/// Returns a span with the trace context of a request that has a single valid `traceparent`
/// header, or a disabled span.
pub(super) fn new(headers: &HeaderMap) -> Span {
    // `traceparent` is not a list header, so several values are invalid
    let mut values = headers.get_all("traceparent").iter();
    let traceparent = match (values.next(), values.next()) {
        (Some(v), None) => v.to_str().ok().and_then(parse_traceparent),
        _ => None,
    };

    match traceparent {
        // The `ERROR` level keeps the span enabled at every log level, so it
        // also applies to `WARN` or `ERROR` lines when `info` is filtered out.
        // The span itself is never printed, only its fields.
        Some(tp) => tracing::error_span!(
            target: TARGET,
            NAME,
            trace_id = tp.trace_id,
            span_id = tp.parent_id,
            trace_flags = tp.trace_flags
        ),
        None => Span::none(),
    }
}
