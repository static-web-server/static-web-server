use tokio::time::{sleep, Duration};

#[cfg(unix)]
use {
    crate::Result, futures_util::stream::StreamExt, signal_hook::consts::signal::*,
    signal_hook_tokio::Signals,
};

#[cfg(windows)]
use tokio::sync::oneshot::Receiver;

#[cfg(unix)]
/// It creates a common list of signals stream for `SIGTERM`, `SIGINT` and `SIGQUIT` to be observed.
pub fn create_signals() -> Result<Signals> {
    Ok(Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?)
}

#[cfg(unix)]
/// It waits for a specific type of incoming signals included `ctrl+c`.
pub async fn wait_for_signals(signals: Signals, grace_period_secs: u8) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                // NOTE: for now we don't do something for SIGHUPs
                tracing::debug!("SIGHUP caught, nothing to do about")
            }
            SIGTERM | SIGINT | SIGQUIT => {
                tracing::info!("SIGTERM, SIGINT or SIGQUIT signal caught");
                break;
            }
            _ => unreachable!(),
        }
    }
    // NOTE: once loop above is done then an upstream graceful shutdown should come next.
    delay_graceful_shutdown(grace_period_secs).await;
    tracing::info!("delegating server's graceful shutdown");
}

/// Function intended to delay the server's graceful shutdown providing a grace period in seconds.
async fn delay_graceful_shutdown(grace_period_secs: u8) {
    if grace_period_secs > 0 {
        tracing::info!(
            "grace period of {}s after the SIGTERM started",
            grace_period_secs
        );
        sleep(Duration::from_secs(grace_period_secs.into())).await;
        tracing::info!("grace period has elapsed");
    }
}

#[cfg(windows)]
/// It waits for an incoming `ctrl+c` signal on Windows.
pub async fn wait_for_ctrl_c<F>(
    cancel_recv: Option<Receiver<()>>,
    cancel_fn: F,
    grace_period_secs: u8,
) where
    F: FnOnce(),
{
    if let Some(recv) = cancel_recv {
        if let Err(err) = recv.await {
            tracing::error!("error during cancel recv: {:?}", err)
        }
        cancel_fn()
    } else {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c signal handler");
    }

    delay_graceful_shutdown(grace_period_secs).await;
    tracing::info!("delegating server's graceful shutdown");
}
