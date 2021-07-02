use ctrlc;
use std::sync::mpsc::channel;

use crate::{Context, Result};

/// It waits for a `Ctrl-C` incoming signal.
pub fn wait_for_ctrl_c() -> Result {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("could not send signal on channel"))
        .with_context(|| "error setting Ctrl-C handler".to_owned())?;

    tracing::info!("press Ctrl+C to shutdown server");

    rx.recv()
        .with_context(|| "could not receive signal from channel".to_owned())?;

    tracing::warn!("Ctrl+C signal caught, shutting down server execution");

    Ok(())
}
