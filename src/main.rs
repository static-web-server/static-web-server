extern crate env_logger;
extern crate iron;
extern crate playground_middleware;

use iron::prelude::*;
use playground_middleware::{Cache, GuessContentType, ModifyWith, Prefix, Staticfile};
use std::time::Duration;

const ADDRESS: &'static str = "[::]:8015";

fn main() {
    env_logger::init().expect("Unable to initialize logger");

    let files = Staticfile::new("./public").expect("Directory to serve not found");
    let mut files = Chain::new(files);

    let one_day = Duration::new(60 * 60 * 24, 0);
    let one_year = Duration::new(60 * 60 * 24 * 365, 0);
    let default_content_type = "text/html"
        .parse()
        .expect("Unable to create default content type");

    files.link_after(ModifyWith::new(Cache::new(one_day)));
    files.link_after(Prefix::new(&["assets"], Cache::new(one_year)));
    files.link_after(GuessContentType::new(default_content_type));

    let _server = Iron::new(files)
        .http(ADDRESS)
        .expect("Unable to start server");
    println!("Server listening at {}", ADDRESS);
}
