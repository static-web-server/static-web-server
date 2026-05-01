// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! HTTP/1 server accept-loop.

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::service::RouterService;
use crate::{Context, Result};

#[cfg(any(unix, windows))]
use crate::signals;

use super::ShutdownCtx;

/// Run the HTTP/1 accept loop until a shutdown signal is received.
pub(super) async fn run<F: FnOnce()>(
    tcp_listener: TcpListener,
    router: RouterService,
    addr_str: &str,
    threads: usize,
    ctx: ShutdownCtx,
    _cancel_fn: F,
) -> Result {
    tcp_listener
        .set_nonblocking(true)
        .with_context(|| "failed to set TCP non-blocking mode")?;
    let listener = tokio::net::TcpListener::from_std(tcp_listener)
        .with_context(|| "failed to create tokio::net::TcpListener")?;

    let graceful = GracefulShutdown::new();
    let builder = http1::Builder::new();

    #[cfg(unix)]
    let signals =
        signals::create_signals().with_context(|| "failed to register termination signals")?;
    #[cfg(unix)]
    let handle = signals.handle();

    #[cfg(unix)]
    let cancel_recv = Arc::new(Mutex::new(ctx.cancel_recv));
    #[cfg(unix)]
    let shutdown = signals::wait_for_signals(signals, ctx.grace_period, cancel_recv);
    #[cfg(unix)]
    tokio::pin!(shutdown);

    #[cfg(windows)]
    let shutdown = {
        let ctrl_c_recv = Arc::new(Mutex::new(if !ctx.windows_service {
            Some(ctx.ctrl_c_recv)
        } else {
            None
        }));
        let cancel_recv = Arc::new(Mutex::new(ctx.cancel_recv));
        let grace_period = ctx.grace_period;
        async move { signals::wait_for_ctrl_c_or_cancel(ctrl_c_recv, cancel_recv, grace_period).await }
    };
    #[cfg(windows)]
    tokio::pin!(shutdown);

    tracing::info!(
        parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
        "http1 server is listening on http://{}",
        addr_str
    );
    tracing::info!("press ctrl+c to shut down the server");

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, addr) = match result {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("failed to accept TCP connection: {:?}", e);
                        continue;
                    }
                };
                if let Err(e) = stream.set_nodelay(true) {
                    tracing::warn!("failed to enable TCP_NODELAY for {}: {:?}", addr, e);
                }
                let svc = router.build(Some(addr));
                let conn = builder.serve_connection(TokioIo::new(stream), svc);
                tokio::spawn(graceful.watch(conn));
            }
            _ = &mut shutdown => { break; }
        }
    }

    graceful.shutdown().await;

    #[cfg(unix)]
    handle.close();

    #[cfg(windows)]
    {
        // Abort the Ctrl+C listener task since it could still be blocked on
        // `ctrl_c().await` when shutdown was triggered programmatically (e.g.
        // via `cancel_recv`). Aborting is a no-op if it already completed due
        // to a real Ctrl+C being received.
        ctx.ctrlc_task.abort();
        _cancel_fn();
    }

    tracing::warn!("termination signal caught, shutting down the server execution");
    Ok(())
}
