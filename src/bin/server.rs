#![deny(warnings)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

extern crate static_web_server;

use self::static_web_server::{Result, Server};

fn main() -> Result {
    Server::new().run()?;

    Ok(())
}
