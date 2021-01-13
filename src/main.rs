#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[macro_use]
extern crate log;

use crate::config::Options;
use hyper_native_tls::NativeTlsServer;
use iron::{prelude::*, Listening};
use iron_staticfile_middleware::HttpToHttpsRedirect;
use staticfiles::*;
use structopt::StructOpt;

mod config;
mod error_page;
mod gzip;
mod helpers;
mod logger;
mod signal_manager;
mod staticfiles;

/// Struct for holding a reference to a running iron server instance
#[derive(Debug)]
struct RunningServer {
    listening: Listening,
    server_type: String,
}

fn on_server_running(server_name: &str, running_servers: &[RunningServer]) {
    // Notify when server is running
    running_servers.iter().for_each(|server| {
        logger::log_server(&format!(
            "{} Server ({}) is listening on {}",
            server.server_type, server_name, server.listening.socket
        ))
    });

    // Wait for incoming signals (E.g Ctrl+C (SIGINT), SIGTERM, etc
    signal_manager::wait_for_signal(|sig: signal::Signal| {
        let code = signal_manager::signal_to_int(sig);

        println!();
        warn!("Signal {} caught. Server execution exited.", code);
        std::process::exit(code)
    })
}

fn main() {
    let opts = Options::from_args();

    logger::init(&opts.log_level);

    let addr = &format!("{}{}{}", opts.host, ":", opts.port);

    // Configure & launch the HTTP server

    let files = StaticFiles::new(StaticFilesOptions {
        root_dir: opts.root,
        assets_dir: opts.assets,
        page_50x_path: opts.page50x,
        page_404_path: opts.page404,
        cors_allow_origins: opts.cors_allow_origins,
    });

    let mut running_servers = Vec::new();
    if opts.tls {
        // Launch static HTTPS server
        let ssl = NativeTlsServer::new(opts.tls_pkcs12, &opts.tls_pkcs12_passwd).unwrap();

        match Iron::new(files.handle()).https(addr, ssl) {
            Ok(listening) => running_servers.push(RunningServer {
                listening,
                server_type: "HTTPS".to_string(),
            }),
            Err(err) => panic!("{:?}", err),
        }

        // Launch redirect HTTP server (if requested)
        if let Some(port_redirect) = opts.tls_redirect_from {
            let addr_redirect = &format!("{}{}{}", opts.host, ":", port_redirect);
            let host_redirect = match opts.tls_redirect_host.as_ref() {
                Some(host) => host,
                None => &opts.host,
            };
            let handler =
                Chain::new(HttpToHttpsRedirect::new(&host_redirect, opts.port).permanent());
            match Iron::new(handler).http(addr_redirect) {
                Ok(listening) => running_servers.push(RunningServer {
                    listening,
                    server_type: "Redirect HTTP".to_string(),
                }),
                Err(err) => panic!("{:?}", err),
            }
        }
    } else {
        // Launch static HTTP server
        match Iron::new(files.handle()).http(addr) {
            Ok(listening) => running_servers.push(RunningServer {
                listening,
                server_type: "HTTP".to_string(),
            }),
            Err(err) => panic!("{:?}", err),
        }
    }
    on_server_running(&opts.name, &running_servers);
}

#[cfg(test)]
mod test {
    extern crate hyper;
    extern crate iron_test;
    extern crate tempdir;

    use super::*;

    use std::fs::{DirBuilder, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use self::hyper::header::Headers;
    use self::iron_test::{request, response};
    use self::tempdir::TempDir;
    use iron::headers::{ContentLength, ContentType};
    use iron::status;

    struct TestFilesystemSetup(TempDir);

    impl TestFilesystemSetup {
        fn new() -> Self {
            TestFilesystemSetup(TempDir::new("test").expect("Could not create test directory"))
        }

        fn path(&self) -> &Path {
            self.0.path()
        }

        fn dir(&self, name: &str) -> PathBuf {
            let p = self.path().join(name);
            DirBuilder::new()
                .recursive(true)
                .create(&p)
                .expect("Could not create directory");
            p
        }

        fn file(&self, name: &str, body: Vec<u8>) -> PathBuf {
            let p = self.path().join(name);

            let mut file = File::create(&p).expect("Could not create file");
            file.write_all(&body).expect("Could not write to file");

            p
        }
    }

    #[test]
    fn staticfile_allow_request_methods() {
        let opts = Options::from_args();

        let files = StaticFiles::new(StaticFilesOptions {
            root_dir: opts.root,
            assets_dir: opts.assets,
            page_50x_path: opts.page50x,
            page_404_path: opts.page404,
            cors_allow_origins: "".to_string(),
        });

        let response = request::head("http://127.0.0.1/", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(response.status, Some(status::Ok));

        let response = request::get("http://127.0.0.1/", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(response.status, Some(status::Ok));
    }

    #[test]
    fn staticfile_empty_body_on_head_request() {
        let opts = Options::from_args();

        let files = StaticFiles::new(StaticFilesOptions {
            root_dir: opts.root,
            assets_dir: opts.assets,
            page_50x_path: opts.page50x,
            page_404_path: opts.page404,
            cors_allow_origins: "".to_string(),
        });

        let res = request::head("http://127.0.0.1/", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(res.status, Some(status::Ok));

        let result_body = response::extract_body_to_bytes(res);
        assert_eq!(result_body, vec!());
    }

    #[test]
    fn staticfile_valid_content_length_on_head_request() {
        let root = TestFilesystemSetup::new();
        root.dir("root");
        root.file("index.html", b"<html><h2>hello</h2></html>".to_vec());

        let assets = TestFilesystemSetup::new();
        assets.dir("assets");

        let opts = Options::from_args();

        let files = StaticFiles::new(StaticFilesOptions {
            root_dir: root.path().to_str().unwrap().to_string(),
            assets_dir: assets.path().to_str().unwrap().to_string(),
            page_50x_path: opts.page50x,
            page_404_path: opts.page404,
            cors_allow_origins: "".to_string(),
        });

        let res = request::head("http://127.0.0.1/", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(res.status, Some(status::Ok));

        let content_length = res.headers.get::<ContentLength>().unwrap();

        assert_eq!(content_length.0, 27);
    }

    #[test]
    fn staticfile_zero_content_length_on_404_head_request() {
        let opts = Options::from_args();

        let files = StaticFiles::new(StaticFilesOptions {
            root_dir: opts.root,
            assets_dir: opts.assets,
            page_50x_path: opts.page50x,
            page_404_path: opts.page404,
            cors_allow_origins: "".to_string(),
        });

        let res = request::head("http://127.0.0.1/unknown", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(res.status, Some(status::NotFound));

        let content_length = res.headers.get::<ContentLength>().unwrap();

        assert_eq!(content_length.0, 0);
    }

    #[test]
    fn staticfile_disallow_request_methods() {
        let opts = Options::from_args();

        let files = StaticFiles::new(StaticFilesOptions {
            root_dir: opts.root,
            assets_dir: opts.assets,
            page_50x_path: opts.page50x,
            page_404_path: opts.page404,
            cors_allow_origins: "".to_string(),
        });

        let response = request::post("http://127.0.0.1/", Headers::new(), "", &files.handle())
            .expect("Response was a http error");

        assert_eq!(response.status, Some(status::MethodNotAllowed));

        let response = request::delete("http://127.0.0.1/", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(response.status, Some(status::MethodNotAllowed));

        let response = request::put("http://127.0.0.1/", Headers::new(), "", &files.handle())
            .expect("Response was a http error");

        assert_eq!(response.status, Some(status::MethodNotAllowed));

        let response = request::patch("http://127.0.0.1/", Headers::new(), "", &files.handle())
            .expect("Response was a http error");

        assert_eq!(response.status, Some(status::MethodNotAllowed));

        let response = request::options("http://127.0.0.1/", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(response.status, Some(status::MethodNotAllowed));
    }

    #[test]
    fn staticfile_valid_content_type_for_404() {
        let root = TestFilesystemSetup::new();
        root.dir("root");

        let assets = TestFilesystemSetup::new();
        assets.dir("assets");

        let opts = Options::from_args();

        let files = StaticFiles::new(StaticFilesOptions {
            root_dir: root.path().to_str().unwrap().to_string(),
            assets_dir: assets.path().to_str().unwrap().to_string(),
            page_50x_path: opts.page50x,
            page_404_path: opts.page404,
            cors_allow_origins: "".to_string(),
        });

        let res = request::head("http://127.0.0.1/unknown", Headers::new(), &files.handle())
            .expect("Response was a http error");

        assert_eq!(res.status, Some(status::NotFound));

        let content_type = res.headers.get::<ContentType>().unwrap();

        assert_eq!(
            content_type.0,
            "text/html".parse::<iron::mime::Mime>().unwrap()
        );
    }
}
