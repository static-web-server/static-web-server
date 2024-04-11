// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module provides signals support like `SIGTERM`, `SIGINT` and `SIGQUIT`.
//!

use std::sync::Arc;
use tokio::sync::{watch::Receiver, Mutex};
use tokio::time::{sleep, Duration};

#[cfg(unix)]
use {
    crate::Result, futures_util::stream::StreamExt, signal_hook::consts::signal::*,
    signal_hook_tokio::Signals,
};

#[cfg(unix)]
#[cfg_attr(docsrs, doc(cfg(unix)))]
#[inline]
/// It creates a common list of signals stream for `SIGTERM`, `SIGINT` and `SIGQUIT` to be observed.
pub fn create_signals() -> Result<Signals> {
    Ok(Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?)
}

#[cfg(unix)]
/// It waits for a specific type of incoming signals included `ctrl+c`.
pub async fn wait_for_signals(
    signals: Signals,
    grace_period_secs: u8,
    cancel_recv: Arc<Mutex<Option<Receiver<()>>>>,
) {
    let (first_tx, mut base_rx) = tokio::sync::mpsc::channel(1);
    let last_tx = first_tx.clone();

    tokio::spawn(async move {
        let mut signals = signals.fuse();
        while let Some(signal) = signals.next().await {
            match signal {
                SIGHUP => {
                    // NOTE: for now we don't do something for SIGHUPs
                    tracing::debug!("SIGHUP caught, nothing to do about")
                }
                SIGTERM | SIGINT | SIGQUIT => {
                    tracing::info!("SIGTERM, SIGINT or SIGQUIT signal caught");
                    first_tx.send(()).await.ok();
                    break;
                }
                _ => unreachable!(),
            }
        }
    });

    tokio::spawn(async move {
        if let Some(recv) = &mut *cancel_recv.lock().await {
            recv.changed().await.ok();
            last_tx.send(()).await.ok();
            tracing::info!("signals interrupted manually by cancel_recv");
        }
    });

    base_rx.recv().await.take();

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
#[cfg_attr(docsrs, doc(cfg(windows)))]
/// It waits for an incoming `ctrl+c` signal on Windows.
pub async fn wait_for_ctrl_c(cancel_recv: Arc<Mutex<Option<Receiver<()>>>>, grace_period_secs: u8) {
    if let Some(receiver) = &mut *cancel_recv.lock().await {
        receiver.changed().await.ok();
    }

    delay_graceful_shutdown(grace_period_secs).await;
    tracing::info!("delegating server's graceful shutdown");
}
