// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! HTTP/1 + TLS server accept-loop with optional HTTP to HTTPS redirect.

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::service::RouterService;
use crate::tls::{TlsAcceptor, TlsConfigBuilder};
use crate::{Context, Result};

#[cfg(any(unix, windows))]
use crate::signals;

use super::{ShutdownCtx, TlsConfig, redirect};

/// Run the HTTP/1 + TLS accept loop with an optional HTTP to HTTPS redirect server.
pub(super) async fn run<F: FnOnce()>(
    tcp_listener: TcpListener,
    router: RouterService,
    addr_str: &str,
    threads: usize,
    cfg: TlsConfig,
    ctx: ShutdownCtx,
    _cancel_fn: F,
) -> Result {
    let mut tls = TlsConfigBuilder::new()
        .cert_path(&cfg.tls_cert)
        .key_path(&cfg.tls_key)
        .build()
        .with_context(|| "failed to initialize TLS probably because invalid cert or key file")?;
    tls.alpn_protocols = vec![b"http/1.1".to_vec()];

    let tls_acceptor = TlsAcceptor::new(tls);

    tcp_listener
        .set_nonblocking(true)
        .with_context(|| "failed to set TCP non-blocking mode")?;
    let listener = tokio::net::TcpListener::from_std(tcp_listener)
        .with_context(|| "failed to create tokio::net::TcpListener")?;

    let grace_period = ctx.grace_period;

    #[cfg(unix)]
    let signals =
        signals::create_signals().with_context(|| "failed to register termination signals")?;
    #[cfg(unix)]
    let handle = signals.handle();

    #[cfg(unix)]
    let cancel_recv = Arc::new(Mutex::new(ctx.cancel_recv));
    #[cfg(unix)]
    let redirect_cancel_recv = cancel_recv.clone();

    #[cfg(windows)]
    let cancel_recv = Arc::new(Mutex::new(ctx.cancel_recv));
    #[cfg(windows)]
    let redirect_cancel_recv = cancel_recv.clone();
    #[cfg(windows)]
    let ctrl_c_recv = Arc::new(Mutex::new(if !ctx.windows_service {
        Some(ctx.ctrl_c_recv)
    } else {
        None
    }));
    #[cfg(windows)]
    let redirect_ctrl_c_recv = ctrl_c_recv.clone();

    // Optional HTTP to HTTPS redirect server
    let redirect_task = redirect::maybe_spawn(
        &cfg,
        redirect_cancel_recv,
        #[cfg(windows)]
        redirect_ctrl_c_recv,
        grace_period,
    )?
    .unwrap_or_else(|| tokio::spawn(async { Ok::<_, crate::Error>(()) }));

    let http1_task = tokio::spawn({
        let router = router.clone();
        async move {
            let graceful = GracefulShutdown::new();
            let builder = http1::Builder::new();

            #[cfg(unix)]
            let shutdown = signals::wait_for_signals(signals, grace_period, cancel_recv);
            #[cfg(unix)]
            tokio::pin!(shutdown);

            #[cfg(windows)]
            let shutdown = async move {
                signals::wait_for_ctrl_c_or_cancel(ctrl_c_recv, cancel_recv, grace_period).await;
            };
            #[cfg(windows)]
            tokio::pin!(shutdown);

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
                        let tls_acceptor = tls_acceptor.clone();
                        let svc = router.build(Some(addr));
                        match tls_acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                let conn = builder
                                    .serve_connection(TokioIo::new(tls_stream), svc);
                                tokio::spawn(graceful.watch(conn));
                            }
                            Err(e) => {
                                tracing::debug!(
                                    "TLS handshake error from {}: {:?}", addr, e
                                );
                            }
                        }
                    }
                    _ = &mut shutdown => { break; }
                }
            }

            graceful.shutdown().await;
            Ok::<_, crate::Error>(())
        }
    });

    tracing::info!(
        parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
        "http1 tls server is listening on https://{}",
        addr_str
    );
    tracing::info!("press ctrl+c to shut down the server");

    #[cfg(windows)]
    {
        let (r0, r1) = tokio::try_join!(http1_task, redirect_task)?;
        // NOTE: Abort the Ctrl+C listener task since
        // it could still be blocked on `ctrl_c().await` when
        // shutdown was triggered programmatically (e.g. via `cancel_recv`).
        // Aborting is a no-op if it already completed
        // due to a real Ctrl+C was received.
        ctx.ctrlc_task.abort();
        r0?;
        r1?;
    }
    #[cfg(unix)]
    {
        let (r0, r1) = tokio::try_join!(http1_task, redirect_task)?;
        r0?;
        r1?;
    }

    #[cfg(unix)]
    handle.close();

    #[cfg(windows)]
    _cancel_fn();

    tracing::warn!("termination signal caught, shutting down the server execution");
    Ok(())
}
