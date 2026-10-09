// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Provides logging initialization for the web server.
//!
//! Logs are emitted to stderr by default. When a file path is supplied via
//! [`init`]'s `log_file` parameter (CLI: `--log-file`, env:
//! `SERVER_LOG_FILE`, config: `log-file`) the server additionally streams logs
//! to that file using [`tracing_appender::non_blocking`]. A background thread
//! drains a lock-free queue so the request path is never blocked by disk I/O.
//! ANSI escape codes are always disabled for file output regardless of
//! `--log-with-ansi`.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use tracing::{Event, Level, Subscriber};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    filter::Targets,
    fmt::{
        FmtContext, FormatEvent, FormattedFields,
        format::{FmtSpan, Format, Json, JsonFields, Writer},
        time::{self, FormatTime},
    },
    prelude::*,
    registry::LookupSpan,
};

use crate::{Context, Result, trace_context};

/// Logging output format.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Structured single-line JSON, suited for production and log aggregation.
    Json,
    /// Human-readable text, suited for local development.
    Pretty,
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// Holds the background worker guard for the non-blocking file appender.
///
/// The guard MUST live for the entire program duration; dropping it shuts
/// down the writer thread. Using a `OnceLock` ties its lifetime to the
/// process. The OS reclaims it on exit. Initialization is intentionally a
/// one-shot (matching `tracing`'s global subscriber).
static LOG_FILE_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Logging system initialization.
///
/// Sets up a global tracing subscriber that streams events to stderr and,
/// optionally, to a file. Returns an error if the global subscriber was
/// already initialized or if the log file cannot be opened.
pub fn init(
    log_level: &str,
    log_format: &LogFormat,
    log_with_ansi: bool,
    log_file: Option<&Path>,
) -> Result {
    let log_level = log_level.to_lowercase();

    configure(&log_level, log_format, log_with_ansi, log_file)
        .with_context(|| "failed to initialize logging")?;

    Ok(())
}

/// Initialize logging builder with its level, output format, and optional
/// file destination.
fn configure(
    level: &str,
    format: &LogFormat,
    enable_ansi: bool,
    log_file: Option<&Path>,
) -> Result {
    let level = level
        .parse::<Level>()
        .with_context(|| "failed to parse log level")?;
    // The same level is applied to both stderr and file layers.
    let make_filter = || Targets::default().with_default(level);
    let timer = time::LocalTime::rfc_3339();

    // Build the optional file writer first so any I/O failure (path
    // resolution, file open) is reported before installing the global
    // subscriber.
    let (file_writer, file_guard) = match log_file {
        Some(path) => {
            let (w, guard) = build_file_writer(path)
                .with_context(|| format!("failed to open log file: {}", path.display()))?;
            (Some(w), Some(guard))
        }
        None => (None, None),
    };

    let registry = tracing_subscriber::registry();

    let result = match format {
        LogFormat::Json => {
            let stderr_layer = tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true)
                .with_current_span(false)
                .with_span_list(false)
                .with_timer(timer.clone())
                .map_event_format(TraceContextJson)
                .with_writer(std::io::stderr)
                .with_filter(make_filter());

            let file_layer = file_writer.map(|w| {
                tracing_subscriber::fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(false)
                    .with_span_list(false)
                    .with_timer(timer)
                    .map_event_format(TraceContextJson)
                    .with_ansi(false)
                    .with_writer(w)
                    .with_filter(make_filter())
            });

            registry.with(stderr_layer).with(file_layer).try_init()
        }
        LogFormat::Pretty => {
            let stderr_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_span_events(FmtSpan::CLOSE)
                .with_ansi(enable_ansi)
                .with_timer(timer.clone())
                .with_filter(make_filter());

            let file_layer = file_writer.map(|w| {
                tracing_subscriber::fmt::layer()
                    .with_writer(w)
                    .with_span_events(FmtSpan::CLOSE)
                    .with_ansi(false)
                    .with_timer(timer)
                    .with_filter(make_filter())
            });

            registry.with(stderr_layer).with(file_layer).try_init()
        }
    };

    match result {
        Ok(()) => {
            // Store the guard only after the subscriber is installed.
            if let Some(g) = file_guard {
                let _ = LOG_FILE_GUARD.set(g);
            }
            Ok(())
        }
        Err(err) => Err(anyhow!(err)),
    }
}

/// JSON event format that adds the fields of the enclosing trace context span
/// (see [`trace_context`]) as top-level fields, as the OpenTelemetry
/// [Trace Context in Non-OTLP Log Formats](https://opentelemetry.io/docs/specs/otel/compatibility/logging_trace_context/)
/// specification recommends. The built-in JSON format can only nest span fields
/// under a `span` or `spans` key.
///
/// Events outside of a trace context span are formatted as is.
struct TraceContextJson<T>(Format<Json, T>);

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
                span.name() == trace_context::SPAN_NAME
                    && span.metadata().target() == trace_context::SPAN_TARGET
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

/// Build a non-blocking file writer for the given path.
///
/// Creates any missing parent directories. Uses
/// [`tracing_appender::rolling::never`] (no rotation, single file) wrapped in
/// [`tracing_appender::non_blocking`] so log emission never blocks the request
/// path; a dedicated background thread drains the queue. The returned guard
/// keeps the worker thread alive and must outlive every emitter.
fn build_file_writer(path: &Path) -> Result<(NonBlocking, WorkerGuard)> {
    let (dir, file_name) = split_path(path)?;

    if !dir.as_os_str().is_empty() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create log directory: {}", dir.display()))?;
    }

    // `rolling::never` keeps the file name as-is (no rotation, no date suffix).
    let appender = tracing_appender::rolling::never(dir, file_name);
    // Default buffered-lines limit (128k) trades latency for durability:
    // messages are dropped only under extreme back-pressure, which is the
    // right trade-off for a server hot path.
    let (writer, guard) = tracing_appender::non_blocking(appender);
    Ok((writer, guard))
}

/// Split a log file path into `(directory, file_name)`.
///
/// Returns an error when the path has no file-name component (e.g. ends in a
/// separator) so misconfiguration is reported at startup rather than producing
/// silently broken file output.
fn split_path(path: &Path) -> Result<(&Path, &std::ffi::OsStr)> {
    let file_name = path.file_name().with_context(|| {
        format!(
            "log file path has no file name component: {}",
            path.display()
        )
    })?;
    let dir = path.parent().unwrap_or_else(|| Path::new(""));
    Ok((dir, file_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::handler::RequestHandlerOpts;

    const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    /// Formats the events emitted by `f` with the JSON format used by the
    /// server (`trace_context_format = true`) or with the built-in one, and
    /// returns the output.
    fn capture_json(level: Level, trace_context_format: bool, f: impl FnOnce()) -> String {
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
        let opts = RequestHandlerOpts {
            log_trace_context: true,
            ..Default::default()
        };
        let mut headers = hyper::HeaderMap::new();
        headers.insert("traceparent", TRACEPARENT.parse().expect("header value"));
        let span = trace_context::span(&opts, &headers);
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
            tracing::error_span!(trace_context::SPAN_NAME, trace_id = "x")
                .in_scope(|| tracing::warn!("same name"));
        };
        let out = capture_json(Level::INFO, true, emit);
        assert_eq!(out.lines().count(), 3, "unexpected output:\n{out}");
        assert_eq!(out, capture_json(Level::INFO, false, emit));
    }

    /// `split_path` returns the parent directory and the file-name component
    /// for a well-formed path.
    #[test]
    fn split_path_extracts_dir_and_name() {
        let path = Path::new("/var/log/sws/server.log");
        let (dir, name) = split_path(path).expect("split should succeed");
        assert_eq!(dir, Path::new("/var/log/sws"));
        assert_eq!(name, std::ffi::OsStr::new("server.log"));
    }

    /// A bare file name (no directory) is split into an empty directory
    /// component and the file name itself. Build code skips `create_dir_all`
    /// in that case.
    #[test]
    fn split_path_handles_bare_filename() {
        let path = Path::new("server.log");
        let (dir, name) = split_path(path).expect("split should succeed");
        assert_eq!(dir, Path::new(""));
        assert_eq!(name, std::ffi::OsStr::new("server.log"));
    }

    /// A path with no file-name component (e.g. `/`, `..`, `.`) is rejected
    /// rather than silently producing broken file output. Note that trailing
    /// separators on otherwise valid paths are normalized by `Path::file_name`
    /// on Unix, so `/var/log/sws/` is accepted as `sws` in `/var/log`.
    #[test]
    fn split_path_rejects_paths_without_file_name() {
        for bad in ["/", "..", "."] {
            let res = split_path(Path::new(bad));
            assert!(
                res.is_err(),
                "path {bad:?} should be rejected (no file-name component)"
            );
        }
    }

    /// `build_file_writer` creates missing parent directories on demand.
    #[test]
    fn build_file_writer_creates_parent_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("nested/a/b/server.log");
        let (_writer, _guard) = build_file_writer(&path).expect("build writer");
        assert!(
            tmp.path().join("nested/a/b").is_dir(),
            "parent directories should be created"
        );
    }

    /// The non-blocking file writer end-to-end: install a scoped subscriber
    /// that writes JSON events through `build_file_writer`, emit several log
    /// statements, drop the guard so the worker thread flushes, then verify
    /// the file content.
    ///
    /// Uses `tracing::subscriber::with_default` (scoped, not global) so this
    /// test does not collide with the global subscriber installed by other
    /// integration tests. The scoped subscriber is per-thread, so we emit
    /// from the calling thread only — thread safety of the underlying queue
    /// is the responsibility of `tracing-appender::non_blocking` and is
    /// covered by that crate's own tests.
    #[test]
    fn file_writer_streams_events_to_disk() {
        use std::io::Read;

        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("server.log");

        let (writer, guard) = build_file_writer(&log_path).expect("writer");
        let layer = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_current_span(false)
            .with_span_list(false)
            .with_ansi(false)
            .with_writer(writer)
            .with_filter(Targets::default().with_default(Level::INFO));

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(event = "ready", "first message");
            tracing::info!(event = "ready", "second message");
            for i in 0..8 {
                tracing::info!(worker = i, "burst message");
            }
        });

        // Drop the guard so the background worker flushes and closes.
        drop(guard);

        let mut contents = String::new();
        std::fs::File::open(&log_path)
            .expect("open log file")
            .read_to_string(&mut contents)
            .expect("read log file");

        assert!(
            contents.contains("first message"),
            "expected first message in:\n{contents}"
        );
        assert!(
            contents.contains("second message"),
            "expected second message in:\n{contents}"
        );
        let burst_count = contents.matches("burst message").count();
        assert_eq!(
            burst_count, 8,
            "expected all 8 burst messages; got {burst_count} in:\n{contents}"
        );
        // JSON format check: every non-empty line must be a valid JSON object.
        for line in contents.lines().filter(|l| !l.is_empty()) {
            let parsed: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|err| panic!("line is not JSON ({err}): {line}"));
            assert!(parsed.is_object(), "JSON line must be an object: {line}");
        }
    }
}
