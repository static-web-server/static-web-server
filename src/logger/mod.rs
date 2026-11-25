// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Logging for the web server.
//!
//! [`init`] installs the global `tracing` subscriber, which writes to stderr and, with
//! `--log-file`, to a file as well. ANSI escape codes are always disabled for file output
//! regardless of `--log-with-ansi`. The submodules write the log lines of a request: the
//! access log line, the client addresses it logs, and the W3C trace context of the request.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::Targets,
    fmt::{format::FmtSpan, time},
    prelude::*,
};

use crate::{Context, Result};

pub(crate) mod access;
pub(crate) mod addr;
mod file;
pub(crate) mod trace_context;

use trace_context::json::TraceContextJson;

pub use access::format::AccessLogFormat;

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
            let (w, guard) = file::build_file_writer(path)
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

/// Keep two subscribers that enable nothing registered until the tests end.
/// Tests must call this before registering a subscriber of their own.
///
/// While a single subscriber is registered, tracing-core caches whether a callsite is enabled
/// from the current thread's subscriber alone and without a lock. A thread without a subscriber
/// can then overwrite the cache that registering another thread's subscriber just rebuilt,
/// and that subscriber never sees the callsite's events.
/// With two or more registered, both run under a lock.
/// These two leave the max level at `OFF`, so no callsite is cached before both are registered.
#[cfg(test)]
pub(crate) fn register_test_subscribers() {
    static SUBSCRIBERS: std::sync::LazyLock<[tracing::Dispatch; 2]> =
        std::sync::LazyLock::new(|| {
            [(); 2].map(|()| {
                let subscriber = tracing_subscriber::registry()
                    .with(tracing_subscriber::filter::LevelFilter::OFF);
                tracing::Dispatch::new(subscriber)
            })
        });
    std::sync::LazyLock::force(&SUBSCRIBERS);
}
