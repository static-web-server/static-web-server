// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Server module to construct a multi-threaded HTTP or HTTP/2 web server.

use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::sync::watch::Receiver;

use crate::handler::RequestHandler;
use crate::service::RouterService;
use crate::{Context, Result, Settings};

mod http1;
mod opts;

#[cfg(feature = "http2")]
mod http2;

/// Shutdown context passed to each server sub-module so they can respond to
/// both OS signals and optional programmatic cancellation.
pub(crate) struct ShutdownCtx {
    /// Grace period in seconds before the server is forcefully terminated.
    pub grace_period: u8,
    /// Optional programmatic cancel receiver.
    pub cancel_recv: Option<Receiver<()>>,
    #[cfg(windows)]
    /// Whether the server is running as a Windows service.
    pub windows_service: bool,
    #[cfg(windows)]
    /// Ctrl+C watch receiver used when not running as a Windows service.
    pub ctrl_c_recv: Receiver<()>,
    #[cfg(windows)]
    /// Background task that listens for Ctrl+C and signals the watch channel.
    pub ctrlc_task: tokio::task::JoinHandle<crate::Result<()>>,
}

/// A multi-threaded HTTP or HTTP/2 web server.
pub struct Server {
    opts: Settings,
    worker_threads: usize,
    max_blocking_threads: usize,
}

impl Server {
    /// Create a new multi-threaded server instance.
    pub fn new(opts: Settings) -> Result<Server> {
        let cpus = std::thread::available_parallelism()
            .with_context(|| {
                "unable to get current platform cpus or lack of permissions to query available parallelism"
            })?
            .get();
        let worker_threads = match opts.general.threads_multiplier {
            0 | 1 => cpus,
            n => cpus * n,
        };
        let max_blocking_threads = opts.general.max_blocking_threads;
        Ok(Server {
            opts,
            worker_threads,
            max_blocking_threads,
        })
    }

    /// Run the multi-threaded `Server` as standalone.
    ///
    /// It accepts an optional [`cancel`] parameter to shut down the server
    /// gracefully on demand as a complement to the termination signals handling.
    ///
    /// [`cancel`]: <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html>
    pub fn run_standalone(self, cancel: Option<Receiver<()>>) -> Result {
        self.run_server_on_rt(cancel, || {}, true)
    }

    /// Run the multi-threaded `Server` which will be used by a Windows service.
    ///
    /// It accepts an optional [`cancel`] parameter to shut down the server
    /// gracefully on demand and a `cancel_fn` that will be executed right after
    /// the server shuts down.
    ///
    /// [`cancel`]: <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html>
    #[cfg(windows)]
    pub fn run_as_service<F>(self, cancel: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        self.run_server_on_rt(cancel, cancel_fn, true)
    }

    /// Build and run the multi-threaded `Server` on the Tokio runtime.
    ///
    /// Setting `exit_on_error` to `true` will exit the entire process if
    /// the server fails to start (previous behaviour).
    pub fn run_server_on_rt<F>(
        self,
        cancel_recv: Option<Receiver<()>>,
        cancel_fn: F,
        exit_on_error: bool,
    ) -> Result
    where
        F: FnOnce(),
    {
        tracing::debug!(
            %self.worker_threads,
            "initializing tokio runtime with multi-threaded scheduler"
        );

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.worker_threads)
            .max_blocking_threads(self.max_blocking_threads)
            .thread_name("static-web-server")
            .enable_all()
            .build()?;

        let res = rt.block_on(async {
            tracing::trace!("tokio runtime initialized");
            self.start_server(cancel_recv, cancel_fn).await
        });

        if let Err(err) = &res {
            tracing::error!("server failed to start up: {:?}", err);
            if exit_on_error {
                std::process::exit(1)
            }
        }
        res
    }

    /// Start the Hyper server (HTTP/1 or HTTP/2 + TLS) and block until shutdown.
    ///
    /// This method orchestrates listener creation, options initialization, and
    /// delegates to the appropriate server sub-module.
    async fn start_server<F>(self, cancel_recv: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        tracing::trace!("starting web server");
        tracing::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        let general = self.opts.general;
        let advanced = self.opts.advanced;

        tracing::info!("log level: {}", general.log_level);
        if general.config_file.is_file() {
            tracing::info!("config file used: {}", general.config_file.display());
        } else {
            tracing::debug!(
                "config file path not found or not a regular file: {}",
                general.config_file.display()
            );
        }

        // Determine TCP listener: inherited file descriptor or bound TCP socket
        let (tcp_listener, addr_str) = match general.fd {
            Some(fd) => {
                let listener = ListenFd::from_env()
                    .take_tcp_listener(fd)?
                    .with_context(|| "failed to convert inherited 'fd' into a 'tcp' listener")?;
                tracing::info!(
                    "converted inherited file descriptor {} to a 'tcp' listener",
                    fd
                );
                (listener, format!("@FD({fd})"))
            }
            None => {
                let ip = general
                    .host
                    .parse::<IpAddr>()
                    .with_context(|| format!("failed to parse {} address", general.host))?;
                let addr = SocketAddr::from((ip, general.port));
                let listener = TcpListener::bind(addr)
                    .with_context(|| format!("failed to bind to {addr} address"))?;
                tracing::info!("server bound to tcp socket {}", addr);
                (listener, addr.to_string())
            }
        };

        tracing::info!("runtime worker threads: {}", self.worker_threads);
        tracing::info!(
            "runtime max blocking threads: {}",
            general.max_blocking_threads
        );
        tracing::info!(
            "grace period before graceful shutdown: {}s",
            general.grace_period
        );

        // Initialize request handler options from configuration
        let opts_result = opts::init(&general, advanced)?;
        let router_service = RouterService::new(RequestHandler {
            opts: Arc::from(opts_result.handler_opts),
        });

        // Windows: spawn a background task that bridges Ctrl+C into a watch channel
        #[cfg(windows)]
        let (sender, ctrl_c_recv) = tokio::sync::watch::channel(());
        #[cfg(windows)]
        let windows_service = general.windows_service;
        #[cfg(windows)]
        let ctrlc_task = tokio::spawn(async move {
            if !windows_service {
                tracing::info!("installing graceful shutdown ctrl+c signal handler");
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install ctrl+c signal handler");
                tracing::info!("graceful shutdown ctrl+c signal received");
                let _ = sender.send(());
            }
            Ok::<_, crate::Error>(())
        });

        let ctx = ShutdownCtx {
            grace_period: general.grace_period,
            cancel_recv,
            #[cfg(windows)]
            windows_service,
            #[cfg(windows)]
            ctrl_c_recv,
            #[cfg(windows)]
            ctrlc_task,
        };

        // Dispatch to the HTTP/2 + TLS server when enabled
        #[cfg(feature = "http2")]
        if general.http2 {
            return http2::run(
                tcp_listener,
                router_service,
                &addr_str,
                self.worker_threads,
                http2::Http2Config {
                    tls_cert: general.http2_tls_cert,
                    tls_key: general.http2_tls_key,
                    https_redirect: general.https_redirect,
                    https_redirect_host: general.https_redirect_host,
                    https_redirect_from_port: general.https_redirect_from_port,
                    https_redirect_from_hosts: general.https_redirect_from_hosts,
                    host: general.host,
                    port: general.port,
                    page404: opts_result.page404,
                    page50x: opts_result.page50x,
                },
                ctx,
                cancel_fn,
            )
            .await;
        }

        // Fall back to the plain HTTP/1 server
        http1::run(
            tcp_listener,
            router_service,
            &addr_str,
            self.worker_threads,
            ctx,
            cancel_fn,
        )
        .await
    }
}
