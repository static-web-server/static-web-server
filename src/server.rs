use hyper::server::conn::AddrIncoming;
use hyper::server::Server as HyperServer;
use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::Arc;

use crate::handler::{RequestHandler, RequestHandlerOpts};
use crate::tls::{TlsAcceptor, TlsConfigBuilder};
use crate::{cors, helpers, logger, signals, Settings};
use crate::{service::RouterService, Context, Result};

/// Define a multi-thread HTTP or HTTP/2 web server.
pub struct Server {
    opts: Settings,
    threads: usize,
}

impl Server {
    /// Create new multi-thread server instance.
    pub fn new() -> Result<Server> {
        // Get server config
        let opts = Settings::get()?;

        // Configure number of worker threads
        let cpus = num_cpus::get();
        let threads = match opts.general.threads_multiplier {
            0 | 1 => cpus,
            n => cpus * n,
        };

        Ok(Server { opts, threads })
    }

    /// Build and run the multi-thread `Server`.
    pub fn run(self) -> Result {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.threads)
            .thread_name("static-web-server")
            .enable_all()
            .build()?
            .block_on(async {
                let r = self.start_server().await;
                if r.is_err() {
                    println!("server failed to start up: {:?}", r.unwrap_err());
                    std::process::exit(1)
                }
            });

        Ok(())
    }

    /// Run the inner Hyper `HyperServer` (HTTP1/HTTP2) forever on the current thread
    // using the given configuration.
    async fn start_server(self) -> Result {
        // Config "general" options
        let general = self.opts.general;

        // Config-file "advanced" options
        let advanced_opts = self.opts.advanced;

        // Logging system initialization
        let log_level = &general.log_level.to_lowercase();
        logger::init(log_level).with_context(|| "failed to initialize logging")?;
        tracing::info!("logging level: {}", log_level.to_lowercase());

        // Config file
        if general.config_file.is_some() && general.config_file.is_some() {
            tracing::info!("config file: {}", general.config_file.unwrap().display());
        }

        // Determine TCP listener either file descriptor or TCP socket
        let (tcp_listener, addr_str);
        match general.fd {
            Some(fd) => {
                addr_str = format!("@FD({})", fd);
                tcp_listener = ListenFd::from_env()
                    .take_tcp_listener(fd)?
                    .with_context(|| "failed to convert inherited FD into a TCP listener")?;
                tracing::info!(
                    "converted inherited file descriptor {} to a TCP listener",
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
                    .with_context(|| format!("failed to bind to {} address", addr))?;
                addr_str = addr.to_string();
                tracing::info!("server bound to TCP socket {}", addr_str);
            }
        }

        // Check for a valid root directory
        let root_dir = helpers::get_valid_dirpath(&general.root)
            .with_context(|| "root directory was not found or inaccessible")?;

        // Custom error pages content
        let page404 = helpers::read_file_content(&general.page404);
        let page50x = helpers::read_file_content(&general.page50x);

        // Fallback page content
        let page_fallback = helpers::read_file_content(&general.page_fallback);

        // Number of worker threads option
        let threads = self.threads;
        tracing::info!("runtime worker threads: {}", self.threads);

        // Security Headers option
        let security_headers = general.security_headers;
        tracing::info!("security headers: enabled={}", security_headers);

        // Auto compression based on the `Accept-Encoding` header
        let compression = general.compression;
        tracing::info!("auto compression: enabled={}", compression);

        // Directory listing option
        let dir_listing = general.directory_listing;
        tracing::info!("directory listing: enabled={}", dir_listing);

        // Directory listing order number
        let dir_listing_order = general.directory_listing_order;
        tracing::info!("directory listing order code: {}", dir_listing_order);

        // Cache control headers option
        let cache_control_headers = general.cache_control_headers;
        tracing::info!("cache control headers: enabled={}", cache_control_headers);

        // CORS option
        let cors = cors::new(
            general.cors_allow_origins.trim(),
            general.cors_allow_headers.trim(),
        );

        // `Basic` HTTP Authentication Schema option
        let basic_auth = general.basic_auth.trim().to_owned();
        tracing::info!(
            "basic authentication: enabled={}",
            !general.basic_auth.is_empty()
        );

        // Grace period option
        let grace_period = general.grace_period;
        tracing::info!("grace period before graceful shutdown: {}s", grace_period);

        // Create a service router for Hyper
        let router_service = RouterService::new(RequestHandler {
            opts: Arc::from(RequestHandlerOpts {
                root_dir,
                compression,
                dir_listing,
                dir_listing_order,
                cors,
                security_headers,
                cache_control_headers,
                page404,
                page50x,
                page_fallback,
                basic_auth,
                advanced_opts,
            }),
        });

        // Run the corresponding HTTP Server asynchronously with its given options

        if general.http2 {
            // HTTP/2 + TLS

            tcp_listener
                .set_nonblocking(true)
                .expect("cannot set non-blocking");
            let listener = tokio::net::TcpListener::from_std(tcp_listener)
                .with_context(|| "failed to create tokio::net::TcpListener")?;
            let mut incoming = AddrIncoming::from_listener(listener).with_context(|| {
                "failed to create an AddrIncoming from the current tokio::net::TcpListener"
            })?;
            incoming.set_nodelay(true);

            let tls = TlsConfigBuilder::new()
                .cert_path(&general.http2_tls_cert)
                .key_path(&general.http2_tls_key)
                .build()
                .with_context(|| {
                    "failed to initialize TLS, probably wrong cert/key or file missing"
                })?;

            #[cfg(unix)]
            let signals = signals::create_signals()
                .with_context(|| "failed to register termination signals")?;
            #[cfg(unix)]
            let handle = signals.handle();

            let server =
                HyperServer::builder(TlsAcceptor::new(tls, incoming)).serve(router_service);

            #[cfg(unix)]
            let server =
                server.with_graceful_shutdown(signals::wait_for_signals(signals, grace_period));
            #[cfg(windows)]
            let server = server.with_graceful_shutdown(signals::wait_for_ctrl_c(grace_period));

            tracing::info!(
                parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
                "listening on https://{}",
                addr_str
            );

            tracing::info!("press ctrl+c to shut down the server");

            server.await?;

            #[cfg(unix)]
            handle.close();
        } else {
            // HTTP/1

            #[cfg(unix)]
            let signals = signals::create_signals()
                .with_context(|| "failed to register termination signals")?;
            #[cfg(unix)]
            let handle = signals.handle();

            let server = HyperServer::from_tcp(tcp_listener)
                .unwrap()
                .tcp_nodelay(true)
                .serve(router_service);

            #[cfg(unix)]
            let server =
                server.with_graceful_shutdown(signals::wait_for_signals(signals, grace_period));
            #[cfg(windows)]
            let server = server.with_graceful_shutdown(signals::wait_for_ctrl_c(grace_period));

            tracing::info!(
                parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
                "listening on http://{}",
                addr_str
            );

            tracing::info!("press ctrl+c to shut down the server");

            server.await?;

            #[cfg(unix)]
            handle.close();
        }

        tracing::warn!("termination signal caught, shutting down the server execution");

        Ok(())
    }
}
