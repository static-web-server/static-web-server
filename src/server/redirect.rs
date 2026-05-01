// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Shared HTTP to HTTPS redirect server used by both the HTTP/1+TLS and HTTP/2+TLS modes.

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::watch::Receiver;

use crate::{Context, Result, error_page, https_redirect};

#[cfg(any(unix, windows))]
use crate::signals;

/// Configuration for the HTTP to HTTPS redirect server.
pub(super) struct RedirectConfig {
    /// Resolved redirect options (hostname, port, allowed hosts).
    pub opts: Arc<https_redirect::RedirectOpts>,
    /// Resolved 404 error page path.
    pub page404: PathBuf,
    /// Resolved 50x error page path.
    pub page50x: PathBuf,
    /// Grace period in seconds.
    pub grace_period: u8,
    /// Programmatic cancel receiver shared with the main TLS server.
    pub cancel_recv: Arc<Mutex<Option<Receiver<()>>>>,
    #[cfg(windows)]
    /// Ctrl+C receiver (used when not running as a Windows service; `None` in service mode).
    pub ctrl_c_recv: Arc<Mutex<Option<Receiver<()>>>>,
}

/// Spawn the HTTP to HTTPS redirect server task.
///
/// Returns a [`JoinHandle`] that resolves once the server shuts down. The
/// caller should `tokio::try_join!` it together with the main TLS server task.
pub(super) fn spawn(
    tcp_listener: TcpListener,
    cfg: RedirectConfig,
) -> Result<tokio::task::JoinHandle<crate::Result<()>>> {
    tcp_listener
        .set_nonblocking(true)
        .with_context(|| "failed to set TCP non-blocking mode for redirect listener")?;

    #[cfg(unix)]
    let redirect_signals =
        signals::create_signals().with_context(|| "failed to register termination signals")?;
    #[cfg(unix)]
    let redirect_handle = redirect_signals.handle();

    let handle = tokio::spawn(async move {
        let redirect_listener = tokio::net::TcpListener::from_std(tcp_listener)
            .with_context(|| "failed to create redirect TcpListener")?;
        let graceful = GracefulShutdown::new();
        let http = http1::Builder::new();

        let grace_period = cfg.grace_period;
        let cancel_recv = cfg.cancel_recv;

        #[cfg(unix)]
        let shutdown = signals::wait_for_signals(redirect_signals, grace_period, cancel_recv);
        #[cfg(unix)]
        tokio::pin!(shutdown);

        #[cfg(windows)]
        let ctrl_c_recv = cfg.ctrl_c_recv;
        #[cfg(windows)]
        let shutdown = async move {
            signals::wait_for_ctrl_c_or_cancel(ctrl_c_recv, cancel_recv, grace_period).await;
        };
        #[cfg(windows)]
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                result = redirect_listener.accept() => {
                    let (stream, addr) = match result {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!(
                                "failed to accept redirect TCP connection: {:?}", e
                            );
                            continue;
                        }
                    };
                    if let Err(e) = stream.set_nodelay(true) {
                        tracing::warn!("failed to enable TCP_NODELAY for {}: {:?}", addr, e);
                    }
                    let redirect_opts = cfg.opts.clone();
                    let page404 = cfg.page404.clone();
                    let page50x = cfg.page50x.clone();
                    let svc = hyper::service::service_fn(move |req| {
                        let redirect_opts = redirect_opts.clone();
                        let page404 = page404.clone();
                        let page50x = page50x.clone();
                        async move {
                            let uri = req.uri().clone();
                            let method = req.method().clone();
                            match https_redirect::redirect_to_https(&req, redirect_opts) {
                                Ok(resp) => Ok(resp),
                                Err(status) => error_page::error_response(
                                    &uri, &method, &status, &page404, &page50x,
                                ),
                            }
                        }
                    });
                    let conn = http.serve_connection(TokioIo::new(stream), svc);
                    tokio::spawn(graceful.watch(conn));
                }
                _ = &mut shutdown => { break; }
            }
        }

        graceful.shutdown().await;

        #[cfg(unix)]
        redirect_handle.close();

        Ok::<_, crate::Error>(())
    });

    Ok(handle)
}

/// Build [`RedirectOpts`] and bind the redirect TCP listener from a [`super::TlsConfig`].
///
/// Returns `None` when HTTPS redirect is disabled in the configuration.
pub(super) fn maybe_spawn(
    cfg: &super::TlsConfig,
    cancel_recv: Arc<Mutex<Option<Receiver<()>>>>,
    #[cfg(windows)] ctrl_c_recv: Arc<Mutex<Option<Receiver<()>>>>,
    grace_period: u8,
) -> Result<Option<tokio::task::JoinHandle<crate::Result<()>>>> {
    if !cfg.https_redirect {
        return Ok(None);
    }

    let addr = SocketAddr::new(
        cfg.host
            .parse()
            .with_context(|| format!("failed to parse {} as IP address", cfg.host))?,
        cfg.https_redirect_from_port,
    );

    let tcp_listener = TcpListener::bind(addr)
        .with_context(|| format!("failed to bind redirect listener to {addr}"))?;

    tracing::info!("http1 redirect server is listening on http://{}", addr);

    let allowed_hosts = cfg
        .https_redirect_from_hosts
        .split(',')
        .map(|s| s.trim().to_owned())
        .collect::<Vec<_>>();
    if allowed_hosts.is_empty() {
        bail!("https redirect allowed hosts is empty, provide at least one host or IP");
    }

    let redirect_opts = Arc::new(https_redirect::RedirectOpts {
        https_hostname: cfg.https_redirect_host.clone(),
        https_port: cfg.port,
        allowed_hosts,
    });

    let task = spawn(
        tcp_listener,
        RedirectConfig {
            opts: redirect_opts,
            page404: cfg.page404.clone(),
            page50x: cfg.page50x.clone(),
            grace_period,
            cancel_recv,
            #[cfg(windows)]
            ctrl_c_recv,
        },
    )?;

    Ok(Some(task))
}
