use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::Result;

/// Initialize logging builder with its levels.
pub fn init(level: &str) -> Result {
    let level = level.parse::<Level>()?;
    match tracing_subscriber::fmt()
        .with_max_level(level)
        .with_span_events(FmtSpan::CLOSE)
        .try_init()
    {
        Err(err) => Err(anyhow!(err)),
        _ => Ok(()),
    }
}
