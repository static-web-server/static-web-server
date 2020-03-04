extern crate chrono;
extern crate env_logger;
extern crate flate2;
extern crate hyper_native_tls;
extern crate iron;
extern crate iron_staticfile_middleware;
extern crate nix;
extern crate signal;

#[macro_use]
extern crate log;
extern crate structopt;

mod config;
mod error_page;
mod gzip;
mod logger;
mod signal_manager;
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

fn on_server_running(server_name: &str, proto: &str, addr: &str) {
    // Notify when server is running
    info!(
        "Static {} Server `{}` is running on {}",
        proto, server_name, addr
    );

    // Wait for incoming signals (E.g Ctrl+C (SIGINT), SIGTERM, etc
    signal_manager::wait_for_signal(|sig: signal::Signal| {
        let code = signal_manager::signal_to_int(sig);

        println!();
        warn!("SIGINT {} caught. HTTP Server execution exited.", code);
        std::process::exit(code)
    })
}

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
    let addr = &format!("{}{}{}", opts.host.to_string(), ":", opts.port.to_string());
    let proto = if opts.tls { "HTTPS" } else { "HTTP" };

    // Configure & launch the HTTP server

    let files = StaticFiles::new(StaticFilesOptions {
        root_dir: opts.root,
        assets_dir: opts.assets,
        page_50x_path: opts.page50x,
        page_404_path: opts.page404,
    });

    if opts.tls {
        let ssl = NativeTlsServer::new(opts.tls_pkcs12, &opts.tls_pkcs12_passwd).unwrap();

        match Iron::new(files.handle()).https(addr, ssl) {
            Result::Ok(_) => on_server_running(&opts.name, &proto, addr),
            Result::Err(err) => panic!("{:?}", err),
        }
    } else {
        match Iron::new(files.handle()).http(addr) {
            Result::Ok(_) => on_server_running(&opts.name, &proto, addr),
            Result::Err(err) => panic!("{:?}", err),
        }
    }
}
