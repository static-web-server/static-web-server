use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{Context, Result};

/// Logging system initialization
pub fn init(log_level: &str) -> Result {
    let log_level = log_level.to_lowercase();

    configure(&log_level).with_context(|| "failed to initialize logging")?;

    tracing::info!("logging level: {}", log_level);

    Ok(())
}

/// Initialize logging builder with its levels.
fn configure(level: &str) -> Result {
    let level = level.parse::<Level>()?;

    #[allow(unused)]
    #[cfg(not(windows))]
    let enable_ansi = true;
    #[allow(unused)]
    #[cfg(windows)]
    let enable_ansi = false;

    #[cfg(not(target_os = "wasi"))]
    match tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(level)
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(enable_ansi)
        .try_init()
    {
        Err(err) => Err(anyhow!(err)),
        _ => Ok(()),
    }

    #[cfg(target_os = "wasi")]
    match tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(level)
        .with_span_events(FmtSpan::CLOSE)
        .try_init()
    {
        Err(err) => Err(anyhow!(err)),
        _ => Ok(()),
    }
}
