#![deny(warnings)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[macro_use]
extern crate static_web_server;

use self::static_web_server::{core::config, core::logger, core::rejection};
use structopt::StructOpt;
use warp::Filter;

/// It creates a new server instance with given options.
async fn server(opts: config::Options) {
    logger::init(&opts.log_level);

    let filters = warp::get()
        .and(warp::fs::dir(opts.root))
        .with(warp::compression::gzip())
        .recover(rejection::handle_rejection);

    note!("server is listening on {}:{}", &opts.host, &opts.port);

    let host = opts
        .host
        .parse::<std::net::IpAddr>()
        .expect("not valid IP address");

    warp::serve(filters).run((host, opts.port)).await
}

#[tokio::main(max_threads = 10_000)]
async fn main() {
    server(config::Options::from_args()).await
}
