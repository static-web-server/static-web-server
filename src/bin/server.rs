#![deny(warnings)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

extern crate static_web_server;

use structopt::StructOpt;
use tracing::warn;
use warp::Filter;

use self::static_web_server::core::*;

/// It creates a new server instance with given options.
async fn server(opts: config::Options) -> Result {
    logger::init(&opts.log_level)?;

    // Check a valid root directory
    let root_dir = helpers::get_valid_dirpath(opts.root)?;

    // Read custom error pages content
    let page404 = helpers::read_file_content(opts.page404.as_ref());
    let page50x = helpers::read_file_content(opts.page50x.as_ref());

    // Public HEAD endpoint
    let page404_a = page404.clone();
    let page50x_a = page50x.clone();
    let public_head = warp::head().and(
        warp::fs::dir(root_dir.clone())
            .map(cache::control_headers)
            .with(warp::trace::request())
            .recover(move |r| {
                let page404_a = page404_a.clone();
                let page50x_a = page50x_a.clone();
                async move { rejection::handle_rejection(page404_a, page50x_a, r).await }
            }),
    );

    // Public GET endpoint (default)
    let page404_b = page404.clone();
    let page50x_b = page50x.clone();
    let public_get_default = warp::get().and(
        warp::fs::dir(root_dir.clone())
            .map(cache::control_headers)
            .with(warp::trace::request())
            .recover(move |r| {
                let page404_b = page404_b.clone();
                let page50x_b = page50x_b.clone();
                async move { rejection::handle_rejection(page404_b, page50x_b, r).await }
            }),
    );

    let host = opts.host.parse::<std::net::IpAddr>()?;
    let port = opts.port;

    // Public GET/HEAD endpoints with compression (deflate, gzip, brotli, none)
    let page404_c = page404.clone();
    let page50x_c = page50x.clone();
    match opts.compression.as_ref() {
        "brotli" => {
            tokio::task::spawn(
                warp::serve(
                    public_head.or(warp::get()
                        .and(cache::accept_encoding("br"))
                        .and(
                            warp::fs::dir(root_dir.clone())
                                .map(cache::control_headers)
                                .with(warp::trace::request())
                                .with(warp::compression::brotli(true))
                                .recover(move |r| {
                                    let page404_c = page404_c.clone();
                                    let page50x_c = page50x_c.clone();
                                    async move {
                                        rejection::handle_rejection(page404_c, page50x_c, r).await
                                    }
                                }),
                        )
                        .or(public_get_default)),
                )
                .run((host, port)),
            )
        }
        "deflate" => {
            tokio::task::spawn(
                warp::serve(
                    public_head.or(warp::get()
                        .and(cache::accept_encoding("deflate"))
                        .and(
                            warp::fs::dir(root_dir.clone())
                                .map(cache::control_headers)
                                .with(warp::trace::request())
                                .with(warp::compression::deflate(true))
                                .recover(move |r| {
                                    let page404_c = page404_c.clone();
                                    let page50x_c = page50x_c.clone();
                                    async move {
                                        rejection::handle_rejection(page404_c, page50x_c, r).await
                                    }
                                }),
                        )
                        .or(public_get_default)),
                )
                .run((host, port)),
            )
        }
        "gzip" => {
            tokio::task::spawn(
                warp::serve(
                    public_head.or(warp::get()
                        .and(cache::accept_encoding("gzip"))
                        .and(
                            warp::fs::dir(root_dir.clone())
                                .map(cache::control_headers)
                                .with(warp::trace::request())
                                .with(warp::compression::gzip(true))
                                .recover(move |r| {
                                    let page404_c = page404_c.clone();
                                    let page50x_c = page50x_c.clone();
                                    async move {
                                        rejection::handle_rejection(page404_c, page50x_c, r).await
                                    }
                                }),
                        )
                        .or(public_get_default)),
                )
                .run((host, port)),
            )
        }
        _ => tokio::task::spawn(warp::serve(public_head.or(public_get_default)).run((host, port))),
    };

    signals::wait(|sig: signals::Signal| {
        let code = signals::as_int(sig);
        warn!("Signal {} caught. Server execution exited.", code);
        std::process::exit(code)
    });

    Ok(())
}

fn main() -> Result {
    let opts = config::Options::from_args();
    let n = if opts.threads_multiplier == 0 {
        1
    } else {
        opts.threads_multiplier
    };
    let threads = num_cpus::get() * n;

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()?
        .block_on(async {
            let r = server(opts).await;
            if r.is_err() {
                panic!("Server error: {:?}", r.unwrap_err())
            }
        });

    Ok(())
}
