use std::io;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::Result;

/// Initialize logging builder with its levels.
pub fn init(level: &str) -> Result {
    let level = level.parse::<Level>()?;

    #[cfg(not(windows))]
    let enable_ansi = true;
    #[cfg(windows)]
    let enable_ansi = false;

    match tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_max_level(level)
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(enable_ansi)
        .try_init()
    {
        Err(err) => Err(anyhow!(err)),
        _ => Ok(()),
    }
}
