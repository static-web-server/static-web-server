extern crate chrono;
extern crate env_logger;
extern crate flate2;
extern crate iron;
extern crate iron_staticfile_middleware;

#[macro_use]
extern crate log;
extern crate structopt;

mod config;
mod error_page;
mod gzip;
mod logger;
mod staticfiles;

use crate::config::Options;
use chrono::Local;
use env_logger::Builder;
use iron::prelude::*;
use log::LevelFilter;
use staticfiles::*;
use std::io::Write;
use structopt::StructOpt;

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

    let opts = Options::from_args();

    let files = StaticFiles::new(StaticFilesOptions {
        root_dir: opts.root,
        assets_dir: opts.assets,
        page_50x_path: opts.page50x,
        page_404_path: opts.page404,
    });

    let _address = &format!("{}{}{}", opts.host.to_string(), ":", opts.port.to_string());

    let _server = Iron::new(files.handle())
        .http(_address)
        .expect("Unable to start the HTTP Server");

    info!(
        "Static HTTP Server `{}` is running on {}",
        opts.name, _address
    );
}
