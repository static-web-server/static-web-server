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
use tracing::Level;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    filter::Targets,
    fmt::{format::FmtSpan, time},
    prelude::*,
};

use crate::{Context, Result};

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
                .with_writer(std::io::stderr)
                .with_timer(timer.clone())
                .with_filter(make_filter());

            let file_layer = file_writer.map(|w| {
                tracing_subscriber::fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(false)
                    .with_span_list(false)
                    .with_ansi(false)
                    .with_writer(w)
                    .with_timer(timer)
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
