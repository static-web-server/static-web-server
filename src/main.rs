#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate env_logger;
extern crate envy;
extern crate flate2;
extern crate iron;
extern crate playground_middleware;
extern crate serde;

use chrono::Local;
use env_logger::Builder;
use iron::prelude::*;
use log::LevelFilter;
use playground_middleware::{Cache, GuessContentType, ModifyWith, Prefix, Staticfile};
use std::io::Write;
use std::time::Duration;

#[macro_use]
mod gzip;
mod env;
mod logger;

use crate::env::Config;
use crate::gzip::GzipMiddleware;
use crate::logger::Logger;

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let config = envy::prefixed("SERVER_")
        .from_env::<Config>()
        .expect("Unable to parsing config from env");

    // headers.append_raw("server", config.name.as_bytes().to_vec());

    let _address = &format!(
        "{}{}{}",
        config.host.to_string(),
        ":",
        config.port.to_string()
    );

    let files = Staticfile::new(config.root).expect("Directory to serve not found");
    let mut files = Chain::new(files);

    let one_day = Duration::new(60 * 60 * 24, 0);
    let one_year = Duration::new(60 * 60 * 24 * 365, 0);
    let default_content_type = "text/html"
        .parse()
        .expect("Unable to create default content type");

    files.link_after(ModifyWith::new(Cache::new(one_day)));
    files.link_after(Prefix::new(&[config.assets], Cache::new(one_year)));
    files.link_after(GuessContentType::new(default_content_type));
    files.link_after(GzipMiddleware);
    files.link_after(Logger);

    let _server = Iron::new(files)
        .http(_address)
        .expect("Unable to start server");

    info!("HTTP Server `{}` is running on {}", config.name, _address);
}
