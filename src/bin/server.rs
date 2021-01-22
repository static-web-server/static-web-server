#![deny(warnings)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

extern crate static_web_server;

use self::static_web_server::*;
use structopt::StructOpt;

fn main() -> Result {
    server::Server::new(config::Config::from_args()).run()?;

    Ok(())
}
