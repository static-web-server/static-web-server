// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The JSON event format that writes the trace context fields as top-level fields.

use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        FmtContext, FormatEvent, FormattedFields,
        format::{Format, Json, JsonFields, Writer},
        time::FormatTime,
    },
    registry::LookupSpan,
};

use super::request_span;

/// JSON event format that adds the fields of the enclosing trace context span
/// (see [`request_span`]) as top-level fields, as the OpenTelemetry
/// [Trace Context in Non-OTLP Log Formats](https://opentelemetry.io/docs/specs/otel/compatibility/logging_trace_context/)
/// specification recommends. The built-in JSON format can only nest span fields
/// under a `span` or `spans` key.
///
/// Events outside of a trace context span are formatted as is.
pub(crate) struct TraceContextJson<T>(pub(crate) Format<Json, T>);

impl<S, T> FormatEvent<S, JsonFields> for TraceContextJson<T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, JsonFields>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let Some(span) = ctx.event_scope().and_then(|mut scope| {
            scope.find(|span| {
                span.name() == request_span::NAME
                    && span.metadata().target() == request_span::TARGET
            })
        }) else {
            return self.0.format_event(ctx, writer, event);
        };

        // Format the event first, so that no span extensions guard is held
        // while the inner format reads the span data.
        let mut buf = String::with_capacity(256);
        self.0.format_event(ctx, Writer::new(&mut buf), event)?;

        let extensions = span.extensions();
        // The span fields are stored as a JSON object, e.g. `{"trace_id":"..."}`
        let fields = extensions
            .get::<FormattedFields<JsonFields>>()
            .and_then(|f| f.fields.strip_prefix('{')?.strip_suffix('}'))
            .filter(|f| !f.is_empty());
        // The event is formatted as a JSON object followed by a newline,
        // so the span fields are inserted before its closing brace.
        let (Some(fields), Some(end)) = (fields, buf.rfind('}')) else {
            return writer.write_str(&buf);
        };
        let (head, tail) = buf.split_at(end);
        writer.write_str(head)?;
        if !head.trim_end().ends_with('{') {
            writer.write_char(',')?;
        }
        writer.write_str(fields)?;
        writer.write_str(tail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing::Level;
    use tracing_subscriber::{filter::Targets, prelude::*};

    use crate::logger::register_test_subscribers;

    const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    /// Formats the events emitted by `f` with the JSON format used by the
    /// server (`trace_context_format = true`) or with the built-in one, and
    /// returns the output.
    fn capture_json(level: Level, trace_context_format: bool, f: impl FnOnce()) -> String {
        register_test_subscribers();

        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = {
            let buf = buf.clone();
            move || SharedBuf(buf.clone())
        };
        let filter = Targets::default().with_default(level);
        let layer = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_current_span(false)
            .with_span_list(false)
            .with_timer(());
        if trace_context_format {
            let layer = layer
                .map_event_format(TraceContextJson)
                .with_writer(writer)
                .with_filter(filter);
            tracing::subscriber::with_default(tracing_subscriber::registry().with(layer), f);
        } else {
            let layer = layer.with_writer(writer).with_filter(filter);
            tracing::subscriber::with_default(tracing_subscriber::registry().with(layer), f);
        }
        let out = buf.lock().expect("lock").clone();
        String::from_utf8(out).expect("utf-8 output")
    }

    struct SharedBuf(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for SharedBuf {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("lock").extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn trace_context_span() -> tracing::Span {
        let mut headers = hyper::HeaderMap::new();
        headers.insert("traceparent", TRACEPARENT.parse().expect("header value"));
        let span = request_span::new(&headers);
        assert!(!span.is_none(), "expected a trace context span");
        span
    }

    /// Events inside a trace context span get its fields as top-level JSON
    /// fields, also when the `info` level is filtered out.
    #[test]
    fn json_format_adds_trace_context_fields() {
        let out = capture_json(Level::WARN, true, || {
            trace_context_span().in_scope(|| {
                tracing::info!("filtered out");
                tracing::warn!(uri = "/missing.css", "inside");
            });
            tracing::warn!("outside");
        });

        let lines: Vec<serde_json::Value> = out
            .lines()
            .map(|line| serde_json::from_str(line).expect("valid JSON line"))
            .collect();
        assert_eq!(lines.len(), 2, "unexpected output:\n{out}");

        let inside = &lines[0];
        assert_eq!(inside["message"], "inside");
        assert_eq!(inside["uri"], "/missing.css");
        assert_eq!(inside["trace_id"], "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(inside["span_id"], "00f067aa0ba902b7");
        assert_eq!(inside["trace_flags"], "01");
        assert!(inside.get("span").is_none(), "no nested span object");

        let outside = &lines[1];
        assert_eq!(outside["message"], "outside");
        assert!(outside.get("trace_id").is_none());
        assert!(outside.get("span_id").is_none());
        assert!(outside.get("trace_flags").is_none());
    }

    /// Without a trace context span, the output is the one of the built-in
    /// JSON format, also for events inside other spans.
    #[test]
    fn json_format_without_trace_context_is_unchanged() {
        let emit = || {
            tracing::info!(parent: tracing::info_span!("other", addr = "[::]:8787"), "listening");
            tracing::warn!(uri = "/missing.css", status = 404, "not found");
            // A span with the same name but another target is not a trace context span
            tracing::error_span!(request_span::NAME, trace_id = "x")
                .in_scope(|| tracing::warn!("same name"));
        };
        let out = capture_json(Level::INFO, true, emit);
        assert_eq!(out.lines().count(), 3, "unexpected output:\n{out}");
        assert_eq!(out, capture_json(Level::INFO, false, emit));
    }
}
