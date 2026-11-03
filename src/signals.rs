// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module provides signals support like `SIGTERM`, `SIGINT` and `SIGQUIT`.
//!

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, watch::Receiver};
use tokio::time::{Duration, sleep};

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
    shutting_down: Arc<AtomicBool>,
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
    delay_graceful_shutdown(grace_period_secs, &shutting_down).await;
    tracing::info!("delegating server's graceful shutdown");
}

/// Sets `shutting_down`, then delays the server's graceful shutdown by the grace period in seconds.
async fn delay_graceful_shutdown(grace_period_secs: u8, shutting_down: &AtomicBool) {
    // Fail readiness probes while connections drain.
    shutting_down.store(true, Ordering::Relaxed);
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
pub async fn wait_for_ctrl_c(
    cancel_recv: Arc<Mutex<Option<Receiver<()>>>>,
    grace_period_secs: u8,
    shutting_down: Arc<AtomicBool>,
) {
    if let Some(receiver) = &mut *cancel_recv.lock().await {
        receiver.changed().await.ok();
    }

    delay_graceful_shutdown(grace_period_secs, &shutting_down).await;
    tracing::info!("delegating server's graceful shutdown");
}

#[cfg(windows)]
#[cfg_attr(docsrs, doc(cfg(windows)))]
/// Waits for the first of `ctrl_c_recv` (real Ctrl+C) or `cancel_recv`
/// (programmatic shutdown) to fire.
///
/// Pass `None` inside either `Arc<Mutex<Option<…>>>` to disable that source;
/// the other source then becomes the sole shutdown trigger.
pub async fn wait_for_ctrl_c_or_cancel(
    ctrl_c_recv: Arc<Mutex<Option<Receiver<()>>>>,
    cancel_recv: Arc<Mutex<Option<Receiver<()>>>>,
    grace_period_secs: u8,
    shutting_down: Arc<AtomicBool>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

    let tx1 = tx.clone();
    tokio::spawn(async move {
        if let Some(recv) = &mut *ctrl_c_recv.lock().await {
            recv.changed().await.ok();
            tx1.send(()).await.ok();
        }
    });

    tokio::spawn(async move {
        if let Some(recv) = &mut *cancel_recv.lock().await {
            recv.changed().await.ok();
            tx.send(()).await.ok();
        }
    });

    rx.recv().await;
    delay_graceful_shutdown(grace_period_secs, &shutting_down).await;
    tracing::info!("delegating server's graceful shutdown");
}
