extern crate chrono;
extern crate env_logger;
extern crate flate2;
extern crate iron;
extern crate iron_staticfile_middleware;

#[macro_use]
extern crate log;
extern crate structopt;

use crate::env::Config;
use chrono::Local;
use env_logger::Builder;
use iron::prelude::*;
use log::LevelFilter;
use std::io::Write;
use structopt::StructOpt;

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

    let config = Config::from_args();

    let _address = &format!(
        "{}{}{}",
        config.host.to_string(),
        ":",
        config.port.to_string()
    );

    let _server = Iron::new(staticfiles::handler(config.root, config.assets))
        .http(_address)
        .expect("Unable to start the HTTP Server");

    info!(
        "Static HTTP Server `{}` is running on {}",
        config.name, _address
    );
}
