use std::net::{IpAddr, SocketAddr};
use warp::Filter;

use crate::{cache, config, cors, helpers, logger, rejection, signals, Result};

/// Define a multi-thread HTTP/HTTPS web server.
pub struct Server {
    opts: config::Config,
    threads: usize,
}

impl Server {
    /// Create new multi-thread server instance.
    pub fn new(opts: config::Config) -> Self {
        let n = if opts.threads_multiplier == 0 {
            1
        } else {
            opts.threads_multiplier
        };
        let threads = num_cpus::get() * n;
        Self { opts, threads }
    }

    /// Build and run the `Server` forever on the current thread.
    pub fn run(self) -> Result {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.threads)
            .enable_all()
            .build()?
            .block_on(async {
                let r = self.start_server().await;
                if r.is_err() {
                    panic!("Server error: {:?}", r.unwrap_err())
                }
            });

        Ok(())
    }

    /// Run the inner `Warp` server forever on the current thread.
    async fn start_server(self) -> Result {
        let opts = self.opts;

        logger::init(&opts.log_level)?;

        let ip = opts.host.parse::<IpAddr>()?;
        let addr = SocketAddr::from((ip, opts.port));

        // Check for a valid root directory
        let root_dir = helpers::get_valid_dirpath(opts.root)?;

        // Custom error pages content
        let page404 = helpers::read_file_content(opts.page404.as_ref());
        let page50x = helpers::read_file_content(opts.page50x.as_ref());
        let page404_a = page404.clone();
        let page50x_a = page50x.clone();

        // CORS support
        let (cors_filter, cors_allowed_origins) =
            cors::get_opt_cors_filter(opts.cors_allow_origins.as_ref());

        // Base fs directory filter
        let base_dir_filter = warp::fs::dir(root_dir.clone())
            .map(cache::control_headers)
            .with(warp::trace::request())
            .recover(move |rej| {
                let page404_a = page404_a.clone();
                let page50x_a = page50x_a.clone();
                async move { rejection::handle_rejection(page404_a, page50x_a, rej).await }
            });

        // Public HEAD endpoint
        let public_head = warp::head().and(base_dir_filter.clone());

        // Public GET endpoint (default)
        let public_get_default = warp::get().and(base_dir_filter.clone());

        // Public GET/HEAD endpoints with compression (deflate, gzip, brotli, none)
        match opts.compression.as_ref() {
            "brotli" => tokio::task::spawn(async move {
                let with_dir = warp::fs::dir(root_dir)
                    .map(cache::control_headers)
                    .with(warp::trace::request())
                    .with(warp::compression::brotli(true))
                    .recover(move |rej| {
                        let page404 = page404.clone();
                        let page50x = page50x.clone();
                        async move { rejection::handle_rejection(page404, page50x, rej).await }
                    });

                if let Some(cors_filter) = cors_filter {
                    tracing::info!(
                        cors_enabled = ?true,
                        allowed_origins = ?cors_allowed_origins
                    );
                    warp::serve(
                        public_head.with(cors_filter.clone()).or(warp::get()
                            .and(cache::has_accept_encoding("br"))
                            .and(with_dir)
                            .with(cors_filter.clone())
                            .or(public_get_default.with(cors_filter))),
                    )
                    .run(addr)
                    .await
                } else {
                    warp::serve(
                        public_head.or(warp::get()
                            .and(cache::has_accept_encoding("br"))
                            .and(with_dir)
                            .or(public_get_default)),
                    )
                    .run(addr)
                    .await
                }
            }),
            "deflate" => tokio::task::spawn(async move {
                let with_dir = warp::fs::dir(root_dir)
                    .map(cache::control_headers)
                    .with(warp::trace::request())
                    .with(warp::compression::deflate(true))
                    .recover(move |rej| {
                        let page404 = page404.clone();
                        let page50x = page50x.clone();
                        async move { rejection::handle_rejection(page404, page50x, rej).await }
                    });

                if let Some(cors_filter) = cors_filter {
                    tracing::info!(
                        cors_enabled = ?true,
                        allowed_origins = ?cors_allowed_origins
                    );
                    warp::serve(
                        public_head.with(cors_filter.clone()).or(warp::get()
                            .and(cache::has_accept_encoding("deflate"))
                            .and(with_dir)
                            .with(cors_filter.clone())
                            .or(public_get_default.with(cors_filter))),
                    )
                    .run(addr)
                    .await
                } else {
                    warp::serve(
                        public_head.or(warp::get()
                            .and(cache::has_accept_encoding("deflate"))
                            .and(with_dir)
                            .or(public_get_default)),
                    )
                    .run(addr)
                    .await
                }
            }),
            "gzip" => tokio::task::spawn(async move {
                let with_dir = warp::fs::dir(root_dir)
                    .map(cache::control_headers)
                    .with(warp::trace::request())
                    .with(warp::compression::gzip(true))
                    .recover(move |rej| {
                        let page404 = page404.clone();
                        let page50x = page50x.clone();
                        async move { rejection::handle_rejection(page404, page50x, rej).await }
                    });

                if let Some(cors_filter) = cors_filter {
                    tracing::info!(
                        cors_enabled = ?true,
                        allowed_origins = ?cors_allowed_origins
                    );
                    warp::serve(
                        public_head.with(cors_filter.clone()).or(warp::get()
                            .and(cache::has_accept_encoding("gzip"))
                            .and(with_dir)
                            .with(cors_filter.clone())
                            .or(public_get_default.with(cors_filter))),
                    )
                    .run(addr)
                    .await
                } else {
                    warp::serve(
                        public_head.or(warp::get()
                            .and(cache::has_accept_encoding("gzip"))
                            .and(with_dir)
                            .or(public_get_default)),
                    )
                    .run(addr)
                    .await
                }
            }),
            _ => tokio::task::spawn(async move {
                if let Some(cors_filter) = cors_filter {
                    tracing::info!(
                        cors_enabled = ?true,
                        allowed_origins = ?cors_allowed_origins
                    );
                    let public_get_default = warp::get()
                        .and(base_dir_filter.clone())
                        .with(cors_filter.clone());
                    warp::serve(public_head.or(public_get_default.with(cors_filter)))
                        .run(addr)
                        .await
                } else {
                    warp::serve(public_head.or(public_get_default))
                        .run(addr)
                        .await
                }
            }),
        };

        signals::wait(|sig: signals::Signal| {
            let code = signals::as_int(sig);
            tracing::warn!("Signal {} caught. Server execution exited.", code);
            std::process::exit(code)
        });

        Ok(())
    }
}
