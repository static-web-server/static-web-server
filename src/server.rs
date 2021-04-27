use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use structopt::StructOpt;
use warp::Filter;

use crate::config::{Config, CONFIG};
use crate::{cache, cors, filters, helpers, logger, rejection, signals, Result};

/// Define a multi-thread HTTP or HTTP/2 web server.
pub struct Server {
    threads: usize,
}

impl Server {
    /// Create new multi-thread server instance.
    pub fn new() -> Self {
        // Initialize global config
        CONFIG.set(Config::from_args()).unwrap();
        let opts = Config::global();

        let threads = match opts.threads_multiplier {
            0 | 1 => 1,
            _ => num_cpus::get() * opts.threads_multiplier,
        };
        Self { threads }
    }

    /// Build and run the multi-thread `Server` spawning a new Tokio asynchronous task for it.
    pub fn run(self) -> Result {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("static-web-server")
            .worker_threads(self.threads)
            .build()?
            .block_on(async {
                let r = self.run_server_with_config().await;
                if r.is_err() {
                    panic!("Server error during start up: {:?}", r.unwrap_err())
                }
            });

        Ok(())
    }

    /// Create and run the `Warp` server spawning a new Tokio asynchronous task with the given configuration.
    async fn run_server_with_config(self) -> Result {
        let opts = Config::global();

        logger::init(&opts.log_level)?;

        tracing::info!("runtime worker threads {}", self.threads);
        tracing::info!("runtime max blocking threads {}", self.threads);

        let ip = opts.host.parse::<IpAddr>()?;
        let addr = SocketAddr::from((ip, opts.port));

        // Check for a valid root directory
        let root_dir = helpers::get_valid_dirpath(&opts.root)?;

        // Custom error pages content
        rejection::PAGE_404
            .set(helpers::read_file_content(opts.page404.as_ref()))
            .expect("page 404 is not initialized");
        rejection::PAGE_50X
            .set(helpers::read_file_content(opts.page50x.as_ref()))
            .expect("page 50x is not initialized");

        // CORS support
        let (cors_filter_opt, cors_allowed_origins) =
            cors::get_opt_cors_filter(opts.cors_allow_origins.as_ref());

        // HTTP/2 + TLS
        let http2 = opts.http2;
        let http2_tls_cert_path = &opts.http2_tls_cert;
        let http2_tls_key_path = &opts.http2_tls_key;

        // Spawn a new Tokio asynchronous server task determined by the given compression type (gzip, brotli or none)
        // TODO: this can be simplified by replicating something similar to `warp::compression::auto()` but skipping `deflate
        // see Warp PR #513 https://github.com/seanmonstar/warp/pull/513
        tokio::task::spawn(async move {
            if opts.compression == "brotli" {
                run_server_with_brotli_compression(
                    addr,
                    root_dir,
                    http2,
                    http2_tls_cert_path,
                    http2_tls_key_path,
                    cors_filter_opt,
                    cors_allowed_origins,
                )
                .await;
                return;
            }

            if opts.compression == "gzip" {
                run_server_with_gzip_compression(
                    addr,
                    root_dir,
                    http2,
                    http2_tls_cert_path,
                    http2_tls_key_path,
                    cors_filter_opt,
                    cors_allowed_origins,
                )
                .await;
                return;
            }

            // Fallback HTTP or HTTP/2 server with no compression
            run_server_with_no_compression(
                addr,
                root_dir,
                http2,
                http2_tls_cert_path,
                http2_tls_key_path,
                cors_filter_opt,
                cors_allowed_origins,
            )
            .await
        });

        // Handle incoming signals for Unix-like OS's only
        // TODO: make this more explicit like v1
        // see https://github.com/joseluisq/static-web-server/blob/1.x/src/server.rs#L96
        signals::wait(|sig: signals::Signal| {
            let code = signals::as_int(sig);
            tracing::warn!("Signal {} caught. Server execution exited.", code);
            std::process::exit(code)
        });

        Ok(())
    }
}

/// It creates and starts a Warp HTTP or HTTP/2 server with Brotli compression.
pub async fn run_server_with_brotli_compression(
    addr: SocketAddr,
    root_dir: PathBuf,
    http2: bool,
    http2_tls_cert_path: &'static str,
    http2_tls_key_path: &'static str,
    cors_filter_opt: Option<warp::filters::cors::Builder>,
    cors_allowed_origins: String,
) {
    // Base fs directory filter
    let base_fs_dir_filter = warp::fs::dir(root_dir.clone())
        .map(cache::control_headers)
        .with(warp::trace::request())
        .recover(rejection::handle_rejection);

    // Public HEAD endpoint
    let public_head = warp::head().and(base_fs_dir_filter.clone());

    // Public GET endpoint (default)
    let public_get_default = warp::get().and(base_fs_dir_filter);

    // Current fs directory filter
    let fs_dir_filter = warp::fs::dir(root_dir)
        .map(cache::control_headers)
        .with(warp::compression::brotli(true))
        .with(warp::trace::request())
        .recover(rejection::handle_rejection);

    // Determine CORS filter
    if let Some(cors_filter) = cors_filter_opt {
        tracing::info!(
            cors_enabled = ?true,
            allowed_origins = ?cors_allowed_origins
        );

        let public_head = public_head.with(cors_filter.clone());
        let public_get_default = public_get_default.with(cors_filter.clone());

        let public_get = warp::get()
            .and(filters::has_accept_encoding("br"))
            .and(fs_dir_filter)
            .with(cors_filter.clone());

        let server = warp::serve(public_head.or(public_get).or(public_get_default));

        if http2 {
            server
                .tls()
                .cert_path(http2_tls_cert_path)
                .key_path(http2_tls_key_path)
                .run(addr)
                .await
        } else {
            server.run(addr).await
        }
    } else {
        let public_get = warp::get()
            .and(filters::has_accept_encoding("br"))
            .and(fs_dir_filter);

        let server = warp::serve(public_head.or(public_get).or(public_get_default));

        if http2 {
            server
                .tls()
                .cert_path(http2_tls_cert_path)
                .key_path(http2_tls_key_path)
                .run(addr)
                .await
        } else {
            server.run(addr).await
        }
    }
}

/// It creates and starts a Warp HTTP or HTTP/2 server with GZIP compression.
pub async fn run_server_with_gzip_compression(
    addr: SocketAddr,
    root_dir: PathBuf,
    http2: bool,
    http2_tls_cert_path: &'static str,
    http2_tls_key_path: &'static str,
    cors_filter_opt: Option<warp::filters::cors::Builder>,
    cors_allowed_origins: String,
) {
    // Base fs directory filter
    let base_fs_dir_filter = warp::fs::dir(root_dir.clone())
        .map(cache::control_headers)
        .with(warp::trace::request())
        .recover(rejection::handle_rejection);

    // Public HEAD endpoint
    let public_head = warp::head().and(base_fs_dir_filter.clone());

    // Public GET endpoint (default)
    let public_get_default = warp::get().and(base_fs_dir_filter);

    // Current fs directory filter
    let fs_dir_filter = warp::fs::dir(root_dir)
        .map(cache::control_headers)
        .with(warp::compression::gzip(true))
        .with(warp::trace::request())
        .recover(rejection::handle_rejection);

    // Determine CORS filter
    if let Some(cors_filter) = cors_filter_opt {
        tracing::info!(
            cors_enabled = ?true,
            allowed_origins = ?cors_allowed_origins
        );

        let public_head = public_head.with(cors_filter.clone());
        let public_get_default = public_get_default.with(cors_filter.clone());

        let public_get = warp::get()
            .and(filters::has_accept_encoding("gzip"))
            .and(fs_dir_filter)
            .with(cors_filter.clone());

        let server = warp::serve(public_head.or(public_get).or(public_get_default));

        if http2 {
            server
                .tls()
                .cert_path(http2_tls_cert_path)
                .key_path(http2_tls_key_path)
                .run(addr)
                .await
        } else {
            server.run(addr).await
        }
    } else {
        let public_get = warp::get()
            .and(filters::has_accept_encoding("gzip"))
            .and(fs_dir_filter);

        let server = warp::serve(public_head.or(public_get).or(public_get_default));

        if http2 {
            server
                .tls()
                .cert_path(http2_tls_cert_path)
                .key_path(http2_tls_key_path)
                .run(addr)
                .await
        } else {
            server.run(addr).await
        }
    }
}

/// It creates and starts a Warp HTTP or HTTP/2 server with no compression.
pub async fn run_server_with_no_compression(
    addr: SocketAddr,
    root_dir: PathBuf,
    http2: bool,
    http2_tls_cert_path: &'static str,
    http2_tls_key_path: &'static str,
    cors_filter_opt: Option<warp::filters::cors::Builder>,
    cors_allowed_origins: String,
) {
    // Base fs directory filter
    let base_fs_dir_filter = warp::fs::dir(root_dir.clone())
        .map(cache::control_headers)
        .with(warp::trace::request())
        .recover(rejection::handle_rejection);

    // Public HEAD endpoint
    let public_head = warp::head().and(base_fs_dir_filter.clone());

    // Public GET endpoint (default)
    let public_get_default = warp::get().and(base_fs_dir_filter);

    // Determine CORS filter
    if let Some(cors_filter) = cors_filter_opt {
        tracing::info!(
            cors_enabled = ?true,
            allowed_origins = ?cors_allowed_origins
        );

        let public_get = public_get_default.with(cors_filter.clone());

        let server = warp::serve(public_head.or(public_get));

        if http2 {
            server
                .tls()
                .cert_path(http2_tls_cert_path)
                .key_path(http2_tls_key_path)
                .run(addr)
                .await
        } else {
            server.run(addr).await
        }
    } else {
        let server = warp::serve(public_head.or(public_get_default));

        if http2 {
            server
                .tls()
                .cert_path(http2_tls_cert_path)
                .key_path(http2_tls_key_path)
                .run(addr)
                .await
        } else {
            server.run(addr).await
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
