use crate::error_page::ErrorPage;
use crate::gzip::GzipMiddleware;
use crate::logger::Logger;

use iron::prelude::*;
use iron_staticfile_middleware::{Cache, GuessContentType, ModifyWith, Prefix, Staticfile};
use std::time::Duration;

pub fn handler(root_dir: String, assets_dir: String) -> Chain {
    let mut chain =
        Chain::new(Staticfile::new(root_dir).expect("Directory to serve files was not found"));

    let one_day = Duration::new(60 * 60 * 24, 0);
    let one_year = Duration::new(60 * 60 * 24 * 365, 0);
    let default_content_type = "text/html"
        .parse()
        .expect("Unable to create a default content type header");

    chain.link_after(ModifyWith::new(Cache::new(one_day)));
    chain.link_after(Prefix::new(&[assets_dir], Cache::new(one_year)));
    chain.link_after(GuessContentType::new(default_content_type));
    chain.link_after(GzipMiddleware);
    chain.link_after(Logger);
    chain.link_after(ErrorPage);

    chain
}
