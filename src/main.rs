extern crate chrono;
extern crate env_logger;
extern crate envy;
extern crate flate2;
extern crate iron;
extern crate playground_middleware;
extern crate serde;

#[macro_use]
extern crate log;

use crate::env::Config;
use chrono::Local;
use env_logger::Builder;
use iron::prelude::*;
use log::LevelFilter;
use std::io::Write;

mod env;
mod gzip;
mod logger;
mod staticfiles;

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
        .expect("Unable to parsing the configuration from system env");

    let _address = &format!(
        "{}{}{}",
        config.host.to_string(),
        ":",
        config.port.to_string()
    );

    let _server = Iron::new(staticfiles::handler(config.root, config.assets))
        .http(_address)
        .expect("Unable to start the HTTP Server");

    info!("HTTP Server `{}` is running on {}", config.name, _address);
}
