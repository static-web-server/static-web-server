use ctrlc;
use std::sync::mpsc::channel;

use crate::{Context, Result};

/// It waits for a `Ctrl-C` signal.
pub fn wait_for_ctrl_c() -> Result {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .with_context(|| "Error setting Ctrl-C handler.".to_owned())?;

    tracing::info!("Press Ctrl+C to shutdown server.");

    rx.recv()
        .with_context(|| "Could not receive signal from channel.".to_owned())?;

    tracing::warn!("Ctrl+C signal caught, shutting down server execution.");

    Ok(())
}
