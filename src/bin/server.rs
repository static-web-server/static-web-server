#![deny(warnings)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

extern crate static_web_server;

use structopt::StructOpt;
use tracing::info;
use warp::Filter;

use self::static_web_server::core::*;

/// It creates a new server instance with given options.
async fn server(opts: config::Options) -> Result {
    logger::init(&opts.log_level)?;

    let filters = warp::get()
        .and(warp::fs::dir(opts.root))
        .with(warp::compression::gzip())
        .recover(rejection::handle_rejection)
        .with(warp::trace::request());

    info!("listening on http://[{}]:{}", &opts.host, &opts.port);

    let host = opts.host.parse::<std::net::IpAddr>()?;

    warp::serve(filters).run((host, opts.port)).await;

    Ok(())
}

#[tokio::main(max_threads = 10_000)]
async fn main() -> Result {
    server(config::Options::from_args()).await?;

    Ok(())
}
