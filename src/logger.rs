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
pub fn init(log_level: &str) -> Result {
    let log_level = log_level.to_lowercase();

    configure(&log_level).with_context(|| "failed to initialize logging")?;

    Ok(())
}

/// Initialize logging builder with its levels.
fn configure(level: &str) -> Result {
    let level = level
        .parse::<Level>()
        .with_context(|| "failed to parse log level")?;

    #[cfg(not(windows))]
    let enable_ansi = true;
    #[cfg(windows)]
    let enable_ansi = false;

    let filtered_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(enable_ansi)
        .with_filter(
            Targets::default()
                .with_default(level)
                .with_target("static_web_server::info", Level::INFO)
                .with_target("static_web_server::warn", Level::WARN),
        );

    match tracing_subscriber::registry()
        .with(filtered_layer)
        .try_init()
    {
        Err(err) => Err(anyhow!(err)),
        _ => Ok(()),
    }
}

/// Custom info level macro.
#[macro_export]
macro_rules! server_info {
    ($($arg:tt)*) => {
        tracing::info!(
            target: "static_web_server::info",
            $($arg)*
        )
    };
}

/// Custom warn level macro.
#[macro_export]
macro_rules! server_warn {
    ($($arg:tt)*) => {
        tracing::warn!(
            target: "static_web_server::warn",
            $($arg)*
        )
    };
}
