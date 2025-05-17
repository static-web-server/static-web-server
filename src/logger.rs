// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Provides logging initialization for the web server.
//!

use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt::format::FmtSpan, prelude::*};

use crate::{Context, Result};

/// Logging system initialization
pub fn init(log_level: &str, log_with_ansi: bool) -> Result {
    let log_level = log_level.to_lowercase();

    configure(&log_level, log_with_ansi).with_context(|| "failed to initialize logging")?;

    Ok(())
}

/// Initialize logging builder with its levels.
fn configure(level: &str, enable_ansi: bool) -> Result {
    let level = level
        .parse::<Level>()
        .with_context(|| "failed to parse log level")?;

    let filtered_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(enable_ansi)
        .with_filter(Targets::default().with_default(level));

    match tracing_subscriber::registry()
        .with(filtered_layer)
        .try_init()
    {
        Err(err) => Err(anyhow!(err)),
        _ => Ok(()),
    }
}
