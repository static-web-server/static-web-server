use hyper::server::conn::AddrIncoming;
use hyper::server::Server as HyperServer;
use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};
use structopt::StructOpt;

use crate::handler::{RequestHandler, RequestHandlerOpts};
use crate::tls::{TlsAcceptor, TlsConfigBuilder};
use crate::Result;
use crate::{config::Config, service::RouterService};
use crate::{cors, error_page, helpers, logger, signals};

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
                    panic!("Server error during start up: {:?}", r.unwrap_err())
                }
            });

        Ok(())
    }

    /// Run the inner Hyper `HyperServer` forever on the current thread with the given configuration.
    async fn start_server(self) -> Result {
        let opts = &self.opts;

        logger::init(&opts.log_level)?;

        tracing::info!("runtime worker threads: {}", self.threads);

        let (tcplistener, addr_string);
        match opts.fd {
            Some(fd) => {
                addr_string = format!("@FD({})", fd);
                tcplistener = ListenFd::from_env()
                    .take_tcp_listener(fd)?
                    .expect("Failed to convert inherited FD into a a TCP listener");
                tracing::info!(
                    "Converted inherited file descriptor {} to a TCP listener",
                    fd
                );
            }
            None => {
                let ip = opts.host.parse::<IpAddr>()?;
                let addr = SocketAddr::from((ip, opts.port));
                tcplistener = TcpListener::bind(addr)?;
                addr_string = format!("{:?}", addr);
                tracing::info!("Bound to TCP socket {}", addr_string);
            }
        }

        // Check for a valid root directory
        let root_dir = helpers::get_valid_dirpath(&opts.root)?;

        // Custom error pages content
        error_page::PAGE_404
            .set(helpers::read_file_content(opts.page404.as_ref()))
            .expect("page 404 is not initialized");
        error_page::PAGE_50X
            .set(helpers::read_file_content(opts.page50x.as_ref()))
            .expect("page 50x is not initialized");

        // Security Headers option
        let security_headers = opts.security_headers;
        tracing::info!("security headers: enabled={}", security_headers);

        // Auto compression based on the `Accept-Encoding` header
        let compression = opts.compression;
        tracing::info!("auto compression: enabled={}", compression);

        // Directory listing option
        let dir_listing = opts.directory_listing;
        tracing::info!("directory listing: enabled={}", dir_listing);

        // Spawn a new Tokio asynchronous server task with its given options
        let threads = self.threads;

        // CORS support
        let cors = cors::new(opts.cors_allow_origins.trim().to_owned());

        // Create a service router for Hyper
        let router_service = RouterService::new(RequestHandler {
            opts: RequestHandlerOpts {
                root_dir,
                compression,
                dir_listing,
                cors,
                security_headers,
            },
        });

        if opts.http2 {
            // HTTP/2 + TLS

            let cert_path = opts.http2_tls_cert.clone();
            let key_path = opts.http2_tls_key.clone();

            tokio::task::spawn(async move {
                tcplistener
                    .set_nonblocking(true)
                    .expect("Cannot set non-blocking");
                let listener = tokio::net::TcpListener::from_std(tcplistener)
                    .expect("Failed to create tokio::net::TcpListener");
                let mut incoming = AddrIncoming::from_listener(listener)?;
                incoming.set_nodelay(true);

                let tls = TlsConfigBuilder::new()
                    .cert_path(cert_path)
                    .key_path(key_path)
                    .build()
                    .expect(
                        "error during TLS server initialization, probably cert or key file missing",
                    );

                let server =
                    HyperServer::builder(TlsAcceptor::new(tls, incoming)).serve(router_service);

                tracing::info!(
                    parent: tracing::info_span!("Server::start_server", ?addr_string, ?threads),
                    "listening on https://{}",
                    addr_string
                );

                server.await
            });
        } else {
            // HTTP/1

            tokio::task::spawn(async move {
                let server = HyperServer::from_tcp(tcplistener)
                    .unwrap()
                    .tcp_nodelay(true)
                    .serve(router_service);

                tracing::info!(
                    parent: tracing::info_span!("Server::start_server", ?addr_string, ?threads),
                    "listening on http://{}",
                    addr_string
                );

                server.await
            });
        }

        signals::wait_for_ctrl_c()
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
