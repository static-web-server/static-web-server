use hyper::server::conn::AddrIncoming;
use hyper::server::Server as HyperServer;
use hyper::service::{make_service_fn, service_fn};
use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::Arc;
use structopt::StructOpt;

use crate::config::Config;
use crate::static_files::ArcPath;
use crate::tls::{TlsAcceptor, TlsConfigBuilder};
use crate::Result;
use crate::{error, error_page, handler, helpers, logger};

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
        let root_dir = ArcPath(Arc::new(root_dir));

        // Custom error pages content
        error_page::PAGE_404
            .set(helpers::read_file_content(opts.page404.as_ref()))
            .expect("page 404 is not initialized");
        error_page::PAGE_50X
            .set(helpers::read_file_content(opts.page50x.as_ref()))
            .expect("page 50x is not initialized");

        // TODO: CORS support

        // Auto compression based on the `Accept-Encoding` header
        let compression = opts.compression;
        tracing::info!("auto compression compression: enabled={}", compression);

        // Directory listing option
        let dir_listing = opts.directory_listing;
        tracing::info!("directory listing: enabled={}", dir_listing);

        // Spawn a new Tokio asynchronous server task with its given options
        let threads = self.threads;

        if opts.http2 {
            // HTTP/2 + TLS

            let cert_path = opts.http2_tls_cert.clone();
            let key_path = opts.http2_tls_key.clone();

            tokio::task::spawn(async move {
                let make_service = make_service_fn(move |_| {
                    let root_dir = root_dir.clone();
                    async move {
                        Ok::<_, error::Error>(service_fn(move |req| {
                            let root_dir = root_dir.clone();
                            async move {
                                handler::handle_request(
                                    root_dir.as_ref(),
                                    compression,
                                    dir_listing,
                                    &req,
                                )
                                .await
                            }
                        }))
                    }
                });

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
                    .unwrap();

                let server =
                    HyperServer::builder(TlsAcceptor::new(tls, incoming)).serve(make_service);

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
                let make_service = make_service_fn(move |_| {
                    let root_dir = root_dir.clone();
                    async move {
                        Ok::<_, error::Error>(service_fn(move |req| {
                            let root_dir = root_dir.clone();
                            async move {
                                handler::handle_request(
                                    root_dir.as_ref(),
                                    compression,
                                    dir_listing,
                                    &req,
                                )
                                .await
                            }
                        }))
                    }
                });

                let server = HyperServer::from_tcp(tcplistener)
                    .unwrap()
                    .tcp_nodelay(true)
                    .serve(make_service);

                tracing::info!(
                    parent: tracing::info_span!("Server::start_server", ?addr_string, ?threads),
                    "listening on http://{}",
                    addr_string
                );

                server.await
            });
        }

        handle_signals();

        Ok(())
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(windows))]
/// Handle incoming signals for Unix-like OS's only
fn handle_signals() {
    use crate::signals;

    signals::wait(|sig: signals::Signal| {
        let code = signals::as_int(sig);
        tracing::warn!("Signal {} caught. Server execution exited.", code);
        std::process::exit(code)
    });
}

#[cfg(windows)]
fn handle_signals() {
    // TODO: Windows signals...
}
