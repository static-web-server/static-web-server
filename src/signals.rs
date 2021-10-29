use futures_util::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use crate::Result;

/// It creates a common list of signals stream for `SIGTERM`, `SIGINT` and `SIGQUIT` to be observed.
pub fn create_signals() -> Result<Signals> {
    Ok(Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?)
}

/// It waits for a specific type of incoming signals.
pub async fn wait_for_signals(signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                // Note: for now we don't do something for SIGHUPs
                tracing::debug!("SIGHUP caught, nothing to do about")
            }
            SIGTERM | SIGINT | SIGQUIT => {
                tracing::debug!("an incoming SIGTERM received, SIGINT or SIGQUIT signal, delegating graceful shutdown to server");
                break;
            }
            _ => unreachable!(),
        }
    }
}
