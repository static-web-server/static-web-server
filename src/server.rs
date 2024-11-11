// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Server module intended to construct a multi-threaded HTTP or HTTP/2 web server.
//!

use hyper::server::Server as HyperServer;
use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::sync::{watch::Receiver, Mutex};

use crate::handler::{RequestHandler, RequestHandlerOpts};

#[cfg(all(unix, feature = "experimental"))]
use crate::metrics;
#[cfg(any(unix, windows))]
use crate::signals;

#[cfg(feature = "http2")]
use {
    crate::tls::{TlsAcceptor, TlsConfigBuilder},
    crate::{error, error_page, https_redirect},
    hyper::server::conn::{AddrIncoming, AddrStream},
    hyper::service::{make_service_fn, service_fn},
};

#[cfg(feature = "directory-listing")]
use crate::directory_listing;
#[cfg(feature = "fallback-page")]
use crate::fallback_page;

#[cfg(any(
    feature = "compression",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
))]
use crate::{compression, compression_static};

#[cfg(feature = "basic-auth")]
use crate::basic_auth;

#[cfg(feature = "experimental")]
use crate::mem_cache;

use crate::{
    control_headers, cors, health, helpers, log_addr, maintenance_mode, security_headers, Settings,
};
use crate::{service::RouterService, Context, Result};

/// Define a multi-threaded HTTP or HTTP/2 web server.
pub struct Server {
    opts: Settings,
    worker_threads: usize,
    max_blocking_threads: usize,
}

impl Server {
    /// Create a new multi-threaded server instance.
    pub fn new(opts: Settings) -> Result<Server> {
        // Configure number of worker threads
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
    /// This is a top-level function of [run_server_on_rt](#method.run_server_on_rt).
    ///
    /// It accepts an optional [`cancel`] parameter to shut down the server
    /// gracefully on demand as a complement to the termination signals handling.
    ///
    /// [`cancel`]: <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html>
    pub fn run_standalone(self, cancel: Option<Receiver<()>>) -> Result {
        self.run_server_on_rt(cancel, || {})
    }

    /// Run the multi-threaded `Server` which will be used by a Windows service.
    /// This is a top-level function of [run_server_on_rt](#method.run_server_on_rt).
    ///
    /// It accepts an optional [`cancel`] parameter to shut down the server
    /// gracefully on demand and an optional `cancel_fn` that will be executed
    /// right after the server is shut down.
    ///
    /// [`cancel`]: <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html>
    #[cfg(windows)]
    pub fn run_as_service<F>(self, cancel: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        self.run_server_on_rt(cancel, cancel_fn)
    }

    /// Build and run the multi-threaded `Server` on the Tokio runtime.
    pub fn run_server_on_rt<F>(self, cancel_recv: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        tracing::debug!(%self.worker_threads, "initializing tokio runtime with multi-threaded scheduler");

        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.worker_threads)
            .max_blocking_threads(self.max_blocking_threads)
            .thread_name("static-web-server")
            .enable_all()
            .build()?
            .block_on(async {
                tracing::trace!("tokio runtime initialized");
                if let Err(err) = self.start_server(cancel_recv, cancel_fn).await {
                    tracing::error!("server failed to start up: {:?}", err);
                    std::process::exit(1)
                }
            });

        Ok(())
    }

    /// Run the inner Hyper `HyperServer` (HTTP1/HTTP2) forever on the current thread
    // using the given configuration.
    async fn start_server<F>(self, _cancel_recv: Option<Receiver<()>>, _cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        tracing::trace!("starting web server");
        server_info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        // Config "general" options
        let general = self.opts.general;
        // Config-file "advanced" options
        let advanced_opts = self.opts.advanced;

        server_info!("log level: {}", general.log_level);

        // Config file option
        let config_file = general.config_file;
        if config_file.is_file() {
            server_info!("config file used: {}", config_file.display());
        } else {
            tracing::debug!(
                "config file path not found or not a regular file: {}",
                config_file.display()
            );
        }

        // Determine TCP listener either file descriptor or TCP socket
        let (tcp_listener, addr_str);
        match general.fd {
            Some(fd) => {
                addr_str = format!("@FD({fd})");
                tcp_listener = ListenFd::from_env()
                    .take_tcp_listener(fd)?
                    .with_context(|| "failed to convert inherited 'fd' into a 'tcp' listener")?;
                server_info!(
                    "converted inherited file descriptor {} to a 'tcp' listener",
                    fd
                );
            }
            None => {
                let ip = general
                    .host
                    .parse::<IpAddr>()
                    .with_context(|| format!("failed to parse {} address", general.host))?;
                let addr = SocketAddr::from((ip, general.port));
                tcp_listener = TcpListener::bind(addr)
                    .with_context(|| format!("failed to bind to {addr} address"))?;
                addr_str = addr.to_string();
                server_info!("server bound to tcp socket {}", addr_str);
            }
        }

        // Number of worker threads option
        let threads = self.worker_threads;
        server_info!("runtime worker threads: {}", threads);

        // Maximum number of blocking threads
        server_info!(
            "runtime max blocking threads: {}",
            general.max_blocking_threads
        );

        // Check for a valid root directory
        let root_dir = helpers::get_valid_dirpath(&general.root)
            .with_context(|| "root directory was not found or inaccessible")?;

        // Custom HTML error page files
        // NOTE: in the case of relative paths, they're joined to the root directory
        let mut page404 = general.page404;
        if page404.is_relative() && !page404.starts_with(&root_dir) {
            page404 = root_dir.join(page404);
        }
        if !page404.is_file() {
            tracing::debug!(
                "404 file path not found or not a regular file: {}",
                page404.display()
            );
        }
        let mut page50x = general.page50x;
        if page50x.is_relative() && !page50x.starts_with(&root_dir) {
            page50x = root_dir.join(page50x);
        }
        if !page50x.is_file() {
            tracing::debug!(
                "50x file path not found or not a regular file: {}",
                page50x.display()
            );
        }

        // Log remote address option
        let log_remote_address = general.log_remote_address;

        // Log the X-Forwarded-For header.
        let log_forwarded_for = general.log_forwarded_for;

        // Trusted IPs for remote addresses.
        let trusted_proxies = general.trusted_proxies;

        // Log redirect trailing slash option
        let redirect_trailing_slash = general.redirect_trailing_slash;
        server_info!(
            "redirect trailing slash: enabled={}",
            redirect_trailing_slash
        );

        // Ignore hidden files option
        let ignore_hidden_files = general.ignore_hidden_files;
        server_info!("ignore hidden files: enabled={}", ignore_hidden_files);

        // Disable symlinks option
        let disable_symlinks = general.disable_symlinks;
        server_info!("disable symlinks: enabled={}", disable_symlinks);

        // Grace period option
        let grace_period = general.grace_period;
        server_info!("grace period before graceful shutdown: {}s", grace_period);

        // Index files option
        let index_files = general
            .index_files
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect::<Vec<_>>();
        if index_files.is_empty() {
            bail!("index files list is empty, provide at least one index file")
        }
        server_info!("index files: {}", general.index_files);

        // Request handler options, some settings will be filled in by modules
        let mut handler_opts = RequestHandlerOpts {
            root_dir,
            page404: page404.clone(),
            page50x: page50x.clone(),
            log_remote_address,
            log_forwarded_for,
            trusted_proxies,
            redirect_trailing_slash,
            ignore_hidden_files,
            disable_symlinks,
            index_files,
            advanced_opts,
            ..Default::default()
        };

        // Directory listing options
        #[cfg(feature = "directory-listing")]
        directory_listing::init(
            general.directory_listing,
            general.directory_listing_order,
            general.directory_listing_format,
            &mut handler_opts,
        );

        // Fallback page option
        #[cfg(feature = "fallback-page")]
        fallback_page::init(&general.page_fallback, &mut handler_opts);

        // Health endpoint option
        health::init(general.health, &mut handler_opts);

        // Log remote address option
        log_addr::init(general.log_remote_address, &mut handler_opts);

        // Metrics endpoint option (experimental)
        #[cfg(all(unix, feature = "experimental"))]
        metrics::init(general.experimental_metrics, &mut handler_opts);

        // CORS option
        cors::init(
            &general.cors_allow_origins,
            &general.cors_allow_headers,
            &general.cors_expose_headers,
            &mut handler_opts,
        );

        // `Basic` HTTP Authentication Schema option
        #[cfg(feature = "basic-auth")]
        basic_auth::init(&general.basic_auth, &mut handler_opts);

        // Maintenance mode option
        maintenance_mode::init(
            general.maintenance_mode,
            general.maintenance_mode_status,
            general.maintenance_mode_file,
            &mut handler_opts,
        );

        // Check pre-compressed files based on the `Accept-Encoding` header
        #[cfg(any(
            feature = "compression",
            feature = "compression-deflate",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
        ))]
        compression_static::init(general.compression_static, &mut handler_opts);

        // Auto compression based on the `Accept-Encoding` header
        #[cfg(any(
            feature = "compression",
            feature = "compression-deflate",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
        ))]
        compression::init(
            general.compression,
            general.compression_level,
            &mut handler_opts,
        );

        // Cache control headers option
        control_headers::init(general.cache_control_headers, &mut handler_opts);

        // Security Headers option
        security_headers::init(general.security_headers, &mut handler_opts);

        // In-Memory cache option
        #[cfg(feature = "experimental")]
        mem_cache::cache::init(&mut handler_opts)?;

        // Create a service router for Hyper
        let router_service = RouterService::new(RequestHandler {
            opts: Arc::from(handler_opts),
        });

        #[cfg(windows)]
        let (sender, receiver) = tokio::sync::watch::channel(());

        // Windows ctrl+c listening
        #[cfg(windows)]
        let ctrlc_task = tokio::spawn(async move {
            if !general.windows_service {
                server_info!("installing graceful shutdown ctrl+c signal handler");
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install ctrl+c signal handler");
                server_info!("installing graceful shutdown ctrl+c signal handler");
                let _ = sender.send(());
            }
        });

        // Run the corresponding HTTP Server asynchronously with its given options
        #[cfg(feature = "http2")]
        if general.http2 {
            // HTTP to HTTPS redirect option
            let https_redirect = general.https_redirect;
            server_info!("http to https redirect: enabled={}", https_redirect);
            server_info!(
                "http to https redirect host: {}",
                general.https_redirect_host
            );
            server_info!(
                "http to https redirect from port: {}",
                general.https_redirect_from_port
            );
            server_info!(
                "http to https redirect from hosts: {}",
                general.https_redirect_from_hosts
            );

            // HTTP/2 + TLS
            tcp_listener
                .set_nonblocking(true)
                .with_context(|| "failed to set TCP non-blocking mode")?;
            let listener = tokio::net::TcpListener::from_std(tcp_listener)
                .with_context(|| "failed to create tokio::net::TcpListener")?;
            let mut incoming = AddrIncoming::from_listener(listener).with_context(|| {
                "failed to create an AddrIncoming from the current tokio::net::TcpListener"
            })?;
            incoming.set_nodelay(true);

            let http2_tls_cert = match general.http2_tls_cert {
                Some(v) => v,
                _ => bail!("failed to initialize TLS because cert file missing"),
            };
            let http2_tls_key = match general.http2_tls_key {
                Some(v) => v,
                _ => bail!("failed to initialize TLS because key file missing"),
            };

            let tls = TlsConfigBuilder::new()
                .cert_path(&http2_tls_cert)
                .key_path(&http2_tls_key)
                .build()
                .with_context(|| {
                    "failed to initialize TLS probably because invalid cert or key file"
                })?;

            #[cfg(unix)]
            let signals = signals::create_signals()
                .with_context(|| "failed to register termination signals")?;
            #[cfg(unix)]
            let handle = signals.handle();

            let http2_server =
                HyperServer::builder(TlsAcceptor::new(tls, incoming)).serve(router_service);

            #[cfg(unix)]
            let http2_cancel_recv = Arc::new(Mutex::new(_cancel_recv));
            #[cfg(unix)]
            let redirect_cancel_recv = http2_cancel_recv.clone();

            #[cfg(unix)]
            let http2_server = http2_server.with_graceful_shutdown(signals::wait_for_signals(
                signals,
                grace_period,
                http2_cancel_recv,
            ));

            #[cfg(windows)]
            let http2_cancel_recv = Arc::new(Mutex::new(_cancel_recv));
            #[cfg(windows)]
            let redirect_cancel_recv = http2_cancel_recv.clone();

            #[cfg(windows)]
            let http2_ctrlc_recv = Arc::new(Mutex::new(Some(receiver)));
            #[cfg(windows)]
            let redirect_ctrlc_recv = http2_ctrlc_recv.clone();

            #[cfg(windows)]
            let http2_server = http2_server.with_graceful_shutdown(async move {
                if general.windows_service {
                    signals::wait_for_ctrl_c(http2_cancel_recv, grace_period).await;
                } else {
                    signals::wait_for_ctrl_c(http2_ctrlc_recv, grace_period).await;
                }
            });

            server_info!(
                parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
                "http2 server is listening on https://{}",
                addr_str
            );

            // HTTP to HTTPS redirect server
            if general.https_redirect {
                let ip = general
                    .host
                    .parse::<IpAddr>()
                    .with_context(|| format!("failed to parse {} address", general.host))?;
                let addr = SocketAddr::from((ip, general.https_redirect_from_port));
                let tcp_listener = TcpListener::bind(addr)
                    .with_context(|| format!("failed to bind to {addr} address"))?;
                server_info!(
                    parent: tracing::info_span!("Server::start_server", ?addr, ?threads),
                    "http1 redirect server is listening on http://{}",
                    addr
                );
                tcp_listener
                    .set_nonblocking(true)
                    .with_context(|| "failed to set TCP non-blocking mode")?;

                #[cfg(unix)]
                let redirect_signals = signals::create_signals()
                    .with_context(|| "failed to register termination signals")?;
                #[cfg(unix)]
                let redirect_handle = redirect_signals.handle();

                // Allowed redirect hosts
                let redirect_allowed_hosts = general
                    .https_redirect_from_hosts
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .collect::<Vec<_>>();
                if redirect_allowed_hosts.is_empty() {
                    bail!("https redirect allowed hosts is empty, provide at least one host or IP")
                }

                // Redirect options
                let redirect_opts = Arc::new(https_redirect::RedirectOpts {
                    https_hostname: general.https_redirect_host,
                    https_port: general.port,
                    allowed_hosts: redirect_allowed_hosts,
                });

                let server_redirect = HyperServer::from_tcp(tcp_listener)
                    .unwrap()
                    .tcp_nodelay(true)
                    .serve(make_service_fn(move |_: &AddrStream| {
                        let redirect_opts = redirect_opts.clone();
                        let page404 = page404.clone();
                        let page50x = page50x.clone();
                        async move {
                            Ok::<_, error::Error>(service_fn(move |req| {
                                let redirect_opts = redirect_opts.clone();
                                let page404 = page404.clone();
                                let page50x = page50x.clone();
                                async move {
                                    let uri = req.uri();
                                    let method = req.method();
                                    match https_redirect::redirect_to_https(&req, redirect_opts) {
                                        Ok(resp) => Ok(resp),
                                        Err(status) => error_page::error_response(
                                            uri, method, &status, &page404, &page50x,
                                        ),
                                    }
                                }
                            }))
                        }
                    }));

                #[cfg(unix)]
                let server_redirect = server_redirect.with_graceful_shutdown(
                    signals::wait_for_signals(redirect_signals, grace_period, redirect_cancel_recv),
                );
                #[cfg(windows)]
                let server_redirect = server_redirect.with_graceful_shutdown(async move {
                    if general.windows_service {
                        signals::wait_for_ctrl_c(redirect_cancel_recv, grace_period).await;
                    } else {
                        signals::wait_for_ctrl_c(redirect_ctrlc_recv, grace_period).await;
                    }
                });

                // HTTP/2 server task
                let server_task = tokio::spawn(async move {
                    if let Err(err) = http2_server.await {
                        tracing::error!("http2 server failed to start up: {:?}", err);
                        std::process::exit(1)
                    }
                });

                // HTTP/1 redirect server task
                let redirect_server_task = tokio::spawn(async move {
                    if let Err(err) = server_redirect.await {
                        tracing::error!("http1 redirect server failed to start up: {:?}", err);
                        std::process::exit(1)
                    }
                });

                server_info!("press ctrl+c to shut down the servers");

                #[cfg(windows)]
                tokio::try_join!(ctrlc_task, server_task, redirect_server_task)?;
                #[cfg(unix)]
                tokio::try_join!(server_task, redirect_server_task)?;

                #[cfg(unix)]
                redirect_handle.close();
            } else {
                server_info!("press ctrl+c to shut down the server");
                http2_server.await?;
            }

            #[cfg(unix)]
            handle.close();

            #[cfg(windows)]
            _cancel_fn();

            server_warn!("termination signal caught, shutting down the server execution");
            return Ok(());
        }

        // HTTP/1

        #[cfg(unix)]
        let signals =
            signals::create_signals().with_context(|| "failed to register termination signals")?;
        #[cfg(unix)]
        let handle = signals.handle();

        tcp_listener
            .set_nonblocking(true)
            .with_context(|| "failed to set TCP non-blocking mode")?;

        let http1_server = HyperServer::from_tcp(tcp_listener)
            .unwrap()
            .tcp_nodelay(true)
            .serve(router_service);

        #[cfg(unix)]
        let http1_cancel_recv = Arc::new(Mutex::new(_cancel_recv));

        #[cfg(unix)]
        let http1_server = http1_server.with_graceful_shutdown(signals::wait_for_signals(
            signals,
            grace_period,
            http1_cancel_recv,
        ));

        #[cfg(windows)]
        let http1_server = http1_server.with_graceful_shutdown(async move {
            let http1_cancel_recv = if general.windows_service {
                // http1_cancel_recv
                Arc::new(Mutex::new(_cancel_recv))
            } else {
                // http1_ctrlc_recv
                Arc::new(Mutex::new(Some(receiver)))
            };
            signals::wait_for_ctrl_c(http1_cancel_recv, grace_period).await;
        });

        server_info!(
            parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
            "http1 server is listening on http://{}",
            addr_str
        );

        server_info!("press ctrl+c to shut down the server");

        #[cfg(unix)]
        http1_server.await?;

        #[cfg(windows)]
        let http1_server_task = tokio::spawn(async move {
            if let Err(err) = http1_server.await {
                tracing::error!("http1 server failed to start up: {:?}", err);
                std::process::exit(1)
            }
        });
        #[cfg(windows)]
        tokio::try_join!(ctrlc_task, http1_server_task)?;

        #[cfg(windows)]
        _cancel_fn();

        #[cfg(unix)]
        handle.close();

        server_warn!("termination signal caught, shutting down the server execution");
        Ok(())
    }
}
