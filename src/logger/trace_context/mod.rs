// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to log the W3C trace context of incoming requests.
//!
//! When enabled, a valid [`traceparent`](https://www.w3.org/TR/trace-context/#traceparent-header)
//! request header opens a `tracing` span around the request handling and the sending of the
//! response body. The span carries the `trace_id`, `span_id` and `trace_flags` fields, so the
//! log lines of the request can be linked to the caller's trace, see
//! [Trace Context in Non-OTLP Log Formats](https://opentelemetry.io/docs/specs/otel/compatibility/logging_trace_context/).
//! No spans are exported and no headers are generated or propagated.
//!
//! The header is taken from any client, so the fields are only reliable behind a proxy that
//! sets or replaces it.

use hyper::header::HeaderMap;
use tracing::Span;

use crate::handler_opts::RequestHandlerOpts;

mod body;
pub(super) mod json;
pub(crate) mod request_span;
mod traceparent;

pub(crate) use body::wrap;

/// Initializes the trace context logging.
pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.log_trace_context = enabled;
    tracing::info!(target: request_span::TARGET, enabled, "log trace context");
}

/// Returns a span with the trace context of the request, or a disabled span
/// when the feature is off or the request has no valid `traceparent` header.
pub(crate) fn span(opts: &RequestHandlerOpts, headers: &HeaderMap) -> Span {
    if !opts.log_trace_context {
        return Span::none();
    }

    request_span::new(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    #[test]
    fn traceparent_span_requires_option_and_single_valid_header() {
        let enabled = RequestHandlerOpts {
            log_trace_context: true,
            ..Default::default()
        };
        let disabled = RequestHandlerOpts::default();
        let mut headers = HeaderMap::new();

        crate::logger::register_test_subscribers();
        tracing::subscriber::with_default(tracing_subscriber::registry(), || {
            assert!(span(&enabled, &headers).is_none());

            headers.insert("traceparent", VALID.parse().unwrap());
            assert!(!span(&enabled, &headers).is_none());
            assert!(span(&disabled, &headers).is_none());

            headers.append("traceparent", VALID.parse().unwrap());
            assert!(span(&enabled, &headers).is_none());

            headers.insert("traceparent", VALID.to_uppercase().parse().unwrap());
            assert!(span(&enabled, &headers).is_none());
        });
    }
}
