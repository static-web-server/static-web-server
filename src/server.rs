use hyper::server::Server as HyperServer;
use hyper::service::{make_service_fn, service_fn};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use structopt::StructOpt;

use crate::{config::Config, error_page};
use crate::{error, helpers, logger, Result};
use crate::{handler, static_files::ArcPath};

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

        tracing::info!("runtime worker threads {}", self.threads);
        tracing::info!("runtime max blocking threads {}", self.threads);

        let ip = opts.host.parse::<IpAddr>()?;
        let addr = SocketAddr::from((ip, opts.port));

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

        // TODO: HTTP/2 + TLS

        // Spawn a new Tokio asynchronous server task with its given options
        tokio::task::spawn(async move {
            let span = tracing::info_span!("Server::run", ?addr, threads = ?self.threads);
            tracing::info!(parent: &span, "listening on http://{}", addr);

            let make_service = make_service_fn(move |_| {
                let root_dir = root_dir.clone();
                async move {
                    Ok::<_, error::Error>(service_fn(move |req| {
                        let root_dir = root_dir.clone();
                        async move { handler::handle_request(root_dir.as_ref(), &req).await }
                    }))
                }
            });

            HyperServer::bind(&addr).serve(make_service).await
        });

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
