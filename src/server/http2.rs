// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! HTTP/2 + TLS server accept-loop with optional HTTP → HTTPS redirect.

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::service::RouterService;
use crate::tls::{TlsAcceptor, TlsConfigBuilder};
use crate::{Context, Result, error_page, https_redirect};

#[cfg(any(unix, windows))]
use crate::signals;

use super::ShutdownCtx;

/// HTTP/2 + TLS and redirect server configuration.
pub(super) struct Http2Config {
    /// Path to the TLS certificate file.
    pub tls_cert: Option<PathBuf>,
    /// Path to the TLS private key file.
    pub tls_key: Option<PathBuf>,
    /// Enable HTTP → HTTPS redirect server.
    pub https_redirect: bool,
    /// Target hostname used in redirect responses.
    pub https_redirect_host: String,
    /// Port the redirect server listens on.
    pub https_redirect_from_port: u16,
    /// Comma-separated list of hosts allowed to be redirected.
    pub https_redirect_from_hosts: String,
    /// Server host address.
    pub host: String,
    /// HTTPS port (used in redirect target URLs).
    pub port: u16,
    /// Resolved 404 error page (used by the redirect server).
    pub page404: PathBuf,
    /// Resolved 50x error page (used by the redirect server).
    pub page50x: PathBuf,
}

/// Run the HTTP/2 + TLS accept loop with an optional HTTP → HTTPS redirect server.
pub(super) async fn run<F: FnOnce()>(
    tcp_listener: TcpListener,
    router: RouterService,
    addr_str: &str,
    threads: usize,
    cfg: Http2Config,
    ctx: ShutdownCtx,
    _cancel_fn: F,
) -> Result {
    // Unpack configuration
    let Http2Config {
        tls_cert,
        tls_key,
        https_redirect,
        https_redirect_host,
        https_redirect_from_port,
        https_redirect_from_hosts,
        host,
        port,
        page404,
        page50x,
    } = cfg;

    let tls_cert =
        tls_cert.ok_or_else(|| anyhow!("failed to initialize TLS because cert file missing"))?;
    let tls_key =
        tls_key.ok_or_else(|| anyhow!("failed to initialize TLS because key file missing"))?;

    tracing::info!("http to https redirect: enabled={}", https_redirect);
    tracing::info!("http to https redirect host: {}", https_redirect_host);
    tracing::info!(
        "http to https redirect from port: {}",
        https_redirect_from_port
    );
    tracing::info!(
        "http to https redirect from hosts: {}",
        https_redirect_from_hosts
    );

    tcp_listener
        .set_nonblocking(true)
        .with_context(|| "failed to set TCP non-blocking mode")?;
    let listener = tokio::net::TcpListener::from_std(tcp_listener)
        .with_context(|| "failed to create tokio::net::TcpListener")?;

    let tls = TlsConfigBuilder::new()
        .cert_path(&tls_cert)
        .key_path(&tls_key)
        .build()
        .with_context(|| "failed to initialize TLS probably because invalid cert or key file")?;
    let tls_acceptor = TlsAcceptor::new(tls);

    let grace_period = ctx.grace_period;

    // Unix: create a signals stream for the HTTP/2 task; keep the handle for cleanup
    #[cfg(unix)]
    let signals =
        signals::create_signals().with_context(|| "failed to register termination signals")?;
    #[cfg(unix)]
    let handle = signals.handle();

    // Wrap cancel_recv in an Arc<Mutex> so it can be shared between tasks
    #[cfg(unix)]
    let http2_cancel_recv = Arc::new(Mutex::new(ctx.cancel_recv));
    #[cfg(unix)]
    let redirect_cancel_recv = http2_cancel_recv.clone();

    #[cfg(windows)]
    let http2_cancel_recv = Arc::new(Mutex::new(ctx.cancel_recv));
    #[cfg(windows)]
    let redirect_cancel_recv = http2_cancel_recv.clone();
    #[cfg(windows)]
    let http2_ctrlc_recv = Arc::new(Mutex::new(Some(ctx.ctrl_c_recv)));
    #[cfg(windows)]
    let redirect_ctrlc_recv = http2_ctrlc_recv.clone();
    #[cfg(windows)]
    let windows_service = ctx.windows_service;

    // HTTP/2 + TLS accept-loop task
    let http2_task = tokio::spawn({
        let tls_acceptor = tls_acceptor.clone();
        let router = router.clone();
        async move {
            let graceful = GracefulShutdown::new();
            let builder =
                hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new());

            #[cfg(unix)]
            let shutdown = signals::wait_for_signals(signals, grace_period, http2_cancel_recv);
            #[cfg(unix)]
            tokio::pin!(shutdown);

            #[cfg(windows)]
            let shutdown = async move {
                if windows_service {
                    signals::wait_for_ctrl_c(http2_cancel_recv, grace_period).await;
                } else {
                    signals::wait_for_ctrl_c(http2_ctrlc_recv, grace_period).await;
                }
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
                                let watcher = graceful.watcher();
                                let builder_clone = builder.clone();
                                tokio::spawn(async move {
                                    let conn = builder_clone
                                        .serve_connection(TokioIo::new(tls_stream), svc)
                                        .into_owned();
                                    if let Err(e) = watcher.watch(conn).await {
                                        tracing::debug!("TLS connection error: {:?}", e);
                                    }
                                });
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
        "http2 server is listening on https://{}",
        addr_str
    );

    // Optional HTTP → HTTPS redirect server task
    let redirect_task = if https_redirect {
        let ip = host
            .parse::<IpAddr>()
            .with_context(|| format!("failed to parse {host} address"))?;
        let addr = SocketAddr::from((ip, https_redirect_from_port));
        let redirect_tcp_listener =
            TcpListener::bind(addr).with_context(|| format!("failed to bind to {addr} address"))?;

        tracing::info!(
            parent: tracing::info_span!("Server::start_server", ?addr, ?threads),
            "http1 redirect server is listening on http://{}",
            addr
        );
        redirect_tcp_listener
            .set_nonblocking(true)
            .with_context(|| "failed to set TCP non-blocking mode")?;

        // Unix: create a separate signals stream for the redirect task
        #[cfg(unix)]
        let redirect_signals =
            signals::create_signals().with_context(|| "failed to register termination signals")?;
        #[cfg(unix)]
        let redirect_handle = redirect_signals.handle();

        let redirect_allowed_hosts = https_redirect_from_hosts
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect::<Vec<_>>();
        if redirect_allowed_hosts.is_empty() {
            bail!("https redirect allowed hosts is empty, provide at least one host or IP")
        }

        let redirect_opts = Arc::new(https_redirect::RedirectOpts {
            https_hostname: https_redirect_host,
            https_port: port,
            allowed_hosts: redirect_allowed_hosts,
        });

        tokio::spawn(async move {
            let redirect_listener = tokio::net::TcpListener::from_std(redirect_tcp_listener)
                .with_context(|| "failed to create redirect TcpListener")?;
            let graceful = GracefulShutdown::new();
            let http = http1::Builder::new();

            #[cfg(unix)]
            let shutdown =
                signals::wait_for_signals(redirect_signals, grace_period, redirect_cancel_recv);
            #[cfg(unix)]
            tokio::pin!(shutdown);

            #[cfg(windows)]
            let shutdown = async move {
                if windows_service {
                    signals::wait_for_ctrl_c(redirect_cancel_recv, grace_period).await;
                } else {
                    signals::wait_for_ctrl_c(redirect_ctrlc_recv, grace_period).await;
                }
            };
            #[cfg(windows)]
            tokio::pin!(shutdown);

            loop {
                tokio::select! {
                    result = redirect_listener.accept() => {
                        let (stream, _addr) = match result {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!(
                                    "failed to accept redirect TCP connection: {:?}", e
                                );
                                continue;
                            }
                        };
                        if let Err(e) = stream.set_nodelay(true) {
                            tracing::warn!(
                                "failed to enable TCP_NODELAY for redirect connection: {:?}",
                                e
                            );
                        }
                        let redirect_opts = redirect_opts.clone();
                        let page404 = page404.clone();
                        let page50x = page50x.clone();
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
                                        &uri,
                                        &method,
                                        &status,
                                        &page404,
                                        &page50x,
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
        })
    } else {
        tokio::spawn(async { Ok::<_, crate::Error>(()) })
    };

    tracing::info!("press ctrl+c to shut down the servers");

    #[cfg(windows)]
    {
        let (r0, r1, r2) = tokio::try_join!(ctx.ctrlc_task, http2_task, redirect_task)?;
        r0?;
        r1?;
        r2?;
    }
    #[cfg(unix)]
    {
        let (r0, r1) = tokio::try_join!(http2_task, redirect_task)?;
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
