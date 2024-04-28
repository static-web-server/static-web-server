// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

/// Graceful shutdown handling
use std::future::Future;
use tokio::sync::watch::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration};

use crate::Error;

#[cfg(unix)]
async fn wait_for_signal() -> Result<(), Error> {
    use futures_util::stream::StreamExt;
    use signal_hook::consts::signal::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
    use signal_hook_tokio::Signals;

    server_info!("installing graceful shutdown signal handler");
    let mut signals = match Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT]) {
        Ok(signals) => signals,
        Err(err) => {
            server_info!("failed to install signal handler");
            return Err(err.into());
        }
    };
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                // We probably should be reloading configuration here
                tracing::debug!("SIGHUP caught and ignored")
            }
            SIGTERM | SIGINT | SIGQUIT => {
                tracing::info!("SIGTERM, SIGINT or SIGQUIT signal caught");
                server_info!("graceful shutdown signal handler triggered");
                return Ok(());
            }
            _ => unreachable!(),
        }
    }

    Err(Error::msg(
        "Signals listener terminated without having received a signal.",
    ))
}

#[cfg(windows)]
async fn wait_for_signal() -> Result<(), Error> {
    server_info!("installing graceful shutdown signal handler");
    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            server_info!("graceful shutdown signal handler triggered");
            Ok(())
        }
        Err(err) => {
            server_info!("failed to install signal handler");
            Err(err.into())
        }
    }
}

#[cfg(not(any(unix, windows)))]
async fn wait_for_signal() -> Result<(), Error> {
    // This should never exit, otherwise the receiver will error out (no
    // senders) and trigger a shutdown.
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

/// Shutdown handler
pub(crate) struct Shutdown {
    sender: Sender<bool>,
    receiver: Receiver<bool>,
}

impl Shutdown {
    /// Creates a new shutdown handler
    pub(crate) fn new() -> Self {
        let (sender, receiver) = channel(false);
        Self { sender, receiver }
    }

    /// Makes shutdown handler listen to a receiver
    pub(crate) fn listen_to(&self, mut receiver: Receiver<()>) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if receiver.changed().await.is_ok() {
                // Ignore send errors, these only occur if all receivers are gone.
                let _ = sender.send(true);
            }
        });
    }

    /// Makes shutdown handler listen to termination signals (Ctrl+C on Windows
    /// or SIGTERM, SIGINT, SIGQUIT on Unix-based systems).
    pub(crate) fn listen_to_signals(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if wait_for_signal().await.is_ok() {
                // Ignore send errors, these only occur if all receivers are gone.
                let _ = sender.send(true);
            }
        });
    }

    pub(crate) fn wait_for_shutdown(&self, grace_period_secs: u8) -> impl Future<Output = ()> {
        let mut receiver = self.receiver.clone();
        async move {
            // Receive errors mean that all senders already shut down, so only
            // wait if we received an actual shutdown signal.
            if receiver.wait_for(|value| *value).await.is_ok() && grace_period_secs > 0 {
                tracing::info!("grace period of {grace_period_secs}s started");
                sleep(Duration::from_secs(grace_period_secs.into())).await;
                tracing::info!("grace period has elapsed");
            }
        }
    }
}
