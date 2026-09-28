// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to log the W3C trace context of incoming requests.
//!
//! When enabled, a valid [`traceparent`](https://www.w3.org/TR/trace-context/#traceparent-header)
//! request header opens a `tracing` span around the request handling. The span carries the
//! `trace_id`, `span_id` and `trace_flags` fields, so the log lines emitted by the request
//! handler can be linked to the caller's trace, see
//! [Trace Context in Non-OTLP Log Formats](https://opentelemetry.io/docs/specs/otel/compatibility/logging_trace_context/).
//! No spans are exported and no headers are generated or propagated.

use hyper::header::HeaderMap;
use tracing::Span;

use crate::handler::RequestHandlerOpts;

/// Name of the span that carries the trace context of a request.
pub(crate) const SPAN_NAME: &str = "trace_context";

/// Target of the span that carries the trace context of a request.
pub(crate) const SPAN_TARGET: &str = module_path!();

/// Length of a version `00` `traceparent` value.
const TRACEPARENT_LEN: usize = 55;

/// Initializes the trace context logging.
pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.log_trace_context = enabled;
    tracing::info!(enabled, "log trace context");
}

/// Returns a span with the trace context of the request, or a disabled span
/// when the feature is off or the request has no valid `traceparent` header.
pub(crate) fn span(opts: &RequestHandlerOpts, headers: &HeaderMap) -> Span {
    if !opts.log_trace_context {
        return Span::none();
    }

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
            target: SPAN_TARGET,
            SPAN_NAME,
            trace_id = tp.trace_id,
            span_id = tp.parent_id,
            trace_flags = tp.trace_flags
        ),
        None => Span::none(),
    }
}

/// Fields of a valid `traceparent` header value.
#[derive(Debug, PartialEq, Eq)]
struct TraceParent<'a> {
    trace_id: &'a str,
    parent_id: &'a str,
    trace_flags: &'a str,
}

/// Parses a `traceparent` header value according to the W3C Trace Context
/// specification. Returns `None` for any invalid value.
///
/// A version higher than `00` is parsed as `00` when the value has at least
/// the `00` length and the extra fields start with a dash. The extra fields
/// are ignored.
fn parse_traceparent(value: &str) -> Option<TraceParent<'_>> {
    let bytes = value.as_bytes();
    // The dashes are checked before any slicing, so every slice below starts
    // and ends at a char boundary.
    if bytes.len() < TRACEPARENT_LEN || bytes[2] != b'-' || bytes[35] != b'-' || bytes[52] != b'-' {
        return None;
    }

    let version = &bytes[..2];
    if !is_lower_hex(version) || version == b"ff" {
        return None;
    }
    let valid_len = if version == b"00" {
        bytes.len() == TRACEPARENT_LEN
    } else {
        bytes.len() == TRACEPARENT_LEN || bytes[TRACEPARENT_LEN] == b'-'
    };
    if !valid_len {
        return None;
    }

    let trace_id = &value[3..35];
    let parent_id = &value[36..52];
    let trace_flags = &value[53..55];
    if !is_non_zero_lower_hex(trace_id.as_bytes())
        || !is_non_zero_lower_hex(parent_id.as_bytes())
        || !is_lower_hex(trace_flags.as_bytes())
    {
        return None;
    }

    Some(TraceParent {
        trace_id,
        parent_id,
        trace_flags,
    })
}

#[inline]
fn is_lower_hex(s: &[u8]) -> bool {
    s.iter().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[inline]
fn is_non_zero_lower_hex(s: &[u8]) -> bool {
    is_lower_hex(s) && s.iter().any(|&b| b != b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    #[test]
    fn traceparent_valid() {
        assert_eq!(
            parse_traceparent(VALID),
            Some(TraceParent {
                trace_id: "4bf92f3577b34da6a3ce929d0e0e4736",
                parent_id: "00f067aa0ba902b7",
                trace_flags: "01",
            })
        );
        let not_sampled = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00";
        assert_eq!(parse_traceparent(not_sampled).unwrap().trace_flags, "00");
    }

    #[test]
    fn traceparent_invalid_values_are_rejected() {
        let invalid = [
            // empty and truncated
            "",
            "00",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0",
            // version `00` with extra data
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-what",
            // forbidden or malformed version
            "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "0g-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "0A-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            // all-zero trace-id and parent-id
            "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",
            // uppercase hex
            "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00F067AA0BA902B7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0A",
            // non-hex characters
            "00-4bf92f3577b34da6a3ce929d0e0e473x-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902bz-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0z",
            // wrong field lengths with the same total length
            "00-4bf92f3577b34da6a3ce929d0e0e47366-0f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b701-",
            // wrong delimiters
            "00_4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736_00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7_01",
        ];
        for value in invalid {
            assert_eq!(parse_traceparent(value), None, "{value:?} must be rejected");
        }
    }

    #[test]
    fn traceparent_non_ascii_is_rejected() {
        // A two-byte char in place of two ASCII bytes keeps the length valid
        let invalid = [
            // The char spans the version and the first dash
            "0\u{e9}4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "\u{e9}-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e47\u{e9}-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-\u{e9}",
        ];
        for value in invalid {
            assert_eq!(parse_traceparent(value), None, "{value:?} must be rejected");
        }
    }

    #[test]
    fn traceparent_future_version() {
        let expected = Some(TraceParent {
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736",
            parent_id: "00f067aa0ba902b7",
            trace_flags: "01",
        });
        // Same layout as `00`
        let same = "cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert_eq!(parse_traceparent(same), expected);
        // Extra fields after a dash are ignored
        let extra = "cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-what-the-future";
        assert_eq!(parse_traceparent(extra), expected);
        // Extra data without a dash after the flags is invalid
        let no_dash = "cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01what";
        assert_eq!(parse_traceparent(no_dash), None);
    }

    #[test]
    fn traceparent_span_requires_option_and_single_valid_header() {
        let enabled = RequestHandlerOpts {
            log_trace_context: true,
            ..Default::default()
        };
        let disabled = RequestHandlerOpts::default();
        let mut headers = HeaderMap::new();

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
