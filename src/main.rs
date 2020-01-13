extern crate chrono;
extern crate env_logger;
extern crate flate2;
extern crate hyper_native_tls;
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
use hyper_native_tls::NativeTlsServer;
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

    let addr = &format!("{}{}{}", opts.host.to_string(), ":", opts.port.to_string());
    let proto = if opts.tls { "HTTPS" } else { "HTTP" };

    let server_info = |server_name: &String, proto: &str, addr: &String| {
        info!(
            "Static {} Server `{}` is running on {}",
            proto, server_name, addr
        );
    };

    if opts.tls {
        let ssl = NativeTlsServer::new(opts.tls_pkcs12, &opts.tls_pkcs12_passwd).unwrap();

        match Iron::new(files.handle()).https(addr, ssl) {
            Result::Ok(_) => server_info(&opts.name, &proto, addr),
            Result::Err(err) => panic!("{:?}", err),
        }
    } else {
        match Iron::new(files.handle()).http(addr) {
            Result::Ok(_) => server_info(&opts.name, &proto, addr),
            Result::Err(err) => panic!("{:?}", err),
        }
    }
}
