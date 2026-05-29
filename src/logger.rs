// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Provides logging initialization for the web server.
//!

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tracing::Level;
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

/// Logging system initialization.
pub fn init(log_level: &str, log_format: &LogFormat, log_with_ansi: bool) -> Result {
    let log_level = log_level.to_lowercase();

    configure(&log_level, log_format, log_with_ansi)
        .with_context(|| "failed to initialize logging")?;

    Ok(())
}

/// Initialize logging builder with its level and output format.
fn configure(level: &str, format: &LogFormat, enable_ansi: bool) -> Result {
    let level = level
        .parse::<Level>()
        .with_context(|| "failed to parse log level")?;
    let filter = Targets::default().with_default(level);
    let timer = time::LocalTime::rfc_3339();

    let result = match format {
        LogFormat::Json => {
            let layer = tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true)
                .with_current_span(false)
                .with_span_list(false)
                .with_writer(std::io::stderr)
                .with_timer(timer)
                .with_filter(filter);
            tracing_subscriber::registry().with(layer).try_init()
        }
        LogFormat::Pretty => {
            let layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_span_events(FmtSpan::CLOSE)
                .with_ansi(enable_ansi)
                .with_timer(timer)
                .with_filter(filter);
            tracing_subscriber::registry().with(layer).try_init()
        }
    };

    match result {
        Err(err) => Err(anyhow!(err)),
        _ => Ok(()),
    }
}
