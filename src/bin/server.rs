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

    let public_head = warp::head().and(
        warp::fs::dir(opts.root.clone())
            .map(cache::control_headers)
            .with(warp::trace::request())
            .recover(rejection::handle_rejection),
    );

    let public_get_default = warp::get().and(
        warp::fs::dir(opts.root.clone())
            .map(cache::control_headers)
            .with(warp::trace::request())
            .recover(rejection::handle_rejection),
    );

    let host = opts.host.parse::<std::net::IpAddr>()?;
    let port = opts.port;

    let accept_encoding = |v: &'static str| warp::header::contains("accept-encoding", v);

    match opts.compression.as_ref() {
        "brotli" => tokio::task::spawn(
            warp::serve(
                public_head.or(warp::get()
                    .and(accept_encoding("br"))
                    .and(
                        warp::fs::dir(opts.root.clone())
                            .map(cache::control_headers)
                            .with(warp::trace::request())
                            .with(warp::compression::brotli(true))
                            .recover(rejection::handle_rejection),
                    )
                    .or(public_get_default)),
            )
            .run((host, port)),
        ),
        "deflate" => tokio::task::spawn(
            warp::serve(
                public_head.or(warp::get()
                    .and(accept_encoding("deflate"))
                    .and(
                        warp::fs::dir(opts.root.clone())
                            .map(cache::control_headers)
                            .with(warp::trace::request())
                            .with(warp::compression::deflate(true))
                            .recover(rejection::handle_rejection),
                    )
                    .or(public_get_default)),
            )
            .run((host, port)),
        ),
        "gzip" => tokio::task::spawn(
            warp::serve(
                public_head.or(warp::get()
                    .and(accept_encoding("gzip"))
                    .and(
                        warp::fs::dir(opts.root.clone())
                            .map(cache::control_headers)
                            .with(warp::trace::request())
                            .with(warp::compression::gzip(true))
                            .recover(rejection::handle_rejection),
                    )
                    .or(public_get_default)),
            )
            .run((host, port)),
        ),
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
