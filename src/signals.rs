#[cfg(not(windows))]
use {
    crate::Result, futures_util::stream::StreamExt, signal_hook::consts::signal::*,
    signal_hook_tokio::Signals,
};

#[cfg(not(windows))]
/// It creates a common list of signals stream for `SIGTERM`, `SIGINT` and `SIGQUIT` to be observed.
pub fn create_signals() -> Result<Signals> {
    Ok(Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?)
}

#[cfg(not(windows))]
/// It waits for a specific type of incoming signals included `ctrl+c`.
pub async fn wait_for_signals(signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                // Note: for now we don't do something for SIGHUPs
                tracing::debug!("SIGHUP caught, nothing to do about")
            }
            SIGTERM | SIGINT | SIGQUIT => {
                tracing::debug!("SIGTERM, SIGINT or SIGQUIT signal received, delegating graceful shutdown to the server");
                break;
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(windows)]
/// It waits for an incoming `ctrl+c` signal on Windows.
pub async fn wait_for_ctrl_c() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl+c signal handler");
}
