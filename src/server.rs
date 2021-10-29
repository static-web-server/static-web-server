use hyper::server::conn::AddrIncoming;
use hyper::server::Server as HyperServer;
use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::Arc;
use structopt::StructOpt;

use crate::handler::{RequestHandler, RequestHandlerOpts};
use crate::tls::{TlsAcceptor, TlsConfigBuilder};
use crate::{config::Config, service::RouterService, Result};
use crate::{cors, helpers, logger, signals};

/// Define a multi-thread HTTP or HTTP/2 web server.
pub struct Server {
    opts: Config,
    threads: usize,
}

impl Server {
    /// Create new multi-thread server instance.
    pub fn new() -> Server {
        // Get server config
        let opts = Config::from_args();

        // Configure number of worker threads
        let cpus = num_cpus::get();
        let threads = match opts.threads_multiplier {
            0 | 1 => cpus,
            n => cpus * n,
        };

        Server { opts, threads }
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
                    panic!("server error during start up: {:?}", r.unwrap_err())
                }
            });

        Ok(())
    }

    /// Run the inner Hyper `HyperServer` (HTTP1/HTTP2) forever on the current thread
    // using the given configuration.
    async fn start_server(self) -> Result {
        let opts = &self.opts;

        // Initialize logging system
        logger::init(&opts.log_level)?;

        // Determine TCP listener either file descriptor or TCP socket
        let (tcp_listener, addr_str);
        match opts.fd {
            Some(fd) => {
                addr_str = format!("@FD({})", fd);
                tcp_listener = ListenFd::from_env()
                    .take_tcp_listener(fd)?
                    .expect("failed to convert inherited FD into a TCP listener");
                tracing::info!(
                    "converted inherited file descriptor {} to a TCP listener",
                    fd
                );
            }
            None => {
                let ip = opts.host.parse::<IpAddr>()?;
                let addr = SocketAddr::from((ip, opts.port));
                tcp_listener = TcpListener::bind(addr)?;
                addr_str = addr.to_string();
                tracing::info!("bound to TCP socket {}", addr_str);
            }
        }

        // Check for a valid root directory
        let root_dir = Arc::new(helpers::get_valid_dirpath(&opts.root)?);

        // Custom error pages content
        let page404 = Arc::from(helpers::read_file_content(opts.page404.as_ref()).as_str());
        let page50x = Arc::from(helpers::read_file_content(opts.page50x.as_ref()).as_str());

        // Number of worker threads option
        let threads = self.threads;
        tracing::info!("runtime worker threads: {}", self.threads);

        // Security Headers option
        let security_headers = opts.security_headers;
        tracing::info!("security headers: enabled={}", security_headers);

        // Auto compression based on the `Accept-Encoding` header
        let compression = opts.compression;
        tracing::info!("auto compression: enabled={}", compression);

        // Directory listing option
        let dir_listing = opts.directory_listing;
        tracing::info!("directory listing: enabled={}", dir_listing);

        // Cache control headers option
        let cache_control_headers = opts.cache_control_headers;
        tracing::info!("cache control headers: enabled={}", cache_control_headers);

        // CORS option
        let cors = cors::new(opts.cors_allow_origins.trim().to_owned());

        // `Basic` HTTP Authentication Schema option
        let basic_auth = opts.basic_auth.trim();
        tracing::info!(
            "basic authentication: enabled={}",
            !self.opts.basic_auth.is_empty()
        );
        let basic_auth = Arc::from(basic_auth);

        // Create a service router for Hyper
        let router_service = RouterService::new(RequestHandler {
            opts: RequestHandlerOpts {
                root_dir,
                compression,
                dir_listing,
                cors,
                security_headers,
                cache_control_headers,
                page404,
                page50x,
                basic_auth,
            },
        });

        // Run the corresponding HTTP Server asynchronously with its given options

        if opts.http2 {
            // HTTP/2 + TLS

            let cert_path = opts.http2_tls_cert.clone();
            let key_path = opts.http2_tls_key.clone();

            tcp_listener
                .set_nonblocking(true)
                .expect("cannot set non-blocking");
            let listener = tokio::net::TcpListener::from_std(tcp_listener)
                .expect("failed to create tokio::net::TcpListener");
            let mut incoming = AddrIncoming::from_listener(listener)?;
            incoming.set_nodelay(true);

            let tls = TlsConfigBuilder::new()
                .cert_path(cert_path)
                .key_path(key_path)
                .build()
                .expect(
                    "error during TLS server initialization, probably cert or key file missing",
                );

            let signals = signals::create_signals()?;
            let handle = signals.handle();

            let server = HyperServer::builder(TlsAcceptor::new(tls, incoming))
                .serve(router_service)
                .with_graceful_shutdown(signals::wait_for_signals(signals));

            tracing::info!(
                parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
                "listening on https://{}",
                addr_str
            );

            tracing::info!("press ctrl+c to shut down the server");

            server.await?;
            handle.close();
        } else {
            // HTTP/1
            let signals = signals::create_signals()?;
            let handle = signals.handle();

            let server = HyperServer::from_tcp(tcp_listener)
                .unwrap()
                .tcp_nodelay(true)
                .serve(router_service)
                .with_graceful_shutdown(signals::wait_for_signals(signals));

            tracing::info!(
                parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
                "listening on http://{}",
                addr_str
            );

            tracing::info!("press ctrl+c to shut down the server");

            server.await?;
            handle.close();
        }

        tracing::warn!("termination signal caught, shutting down the server execution");

        Ok(())
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
