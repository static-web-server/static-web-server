#![deny(warnings)]
#![deny(rust_2018_idioms)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use static_web_server::{server, Options};
use structopt::StructOpt;

fn main() {
    server::run(Options::from_args())
}

#[cfg(test)]
mod test {
    use hyper;
    use iron_test;
    use tempfile;

    use super::*;
    use static_web_server::config::Options;
    use static_web_server::staticfiles::*;

    use std::fs::{DirBuilder, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use self::hyper::header::Headers;
    use self::iron_test::{request, response};
    use self::tempfile::TempDir;
    use iron::headers::{ContentLength, ContentType};
    use iron::status;

    struct TestFilesystemSetup(TempDir);

    impl TestFilesystemSetup {
        fn new() -> Self {
            TestFilesystemSetup(TempDir::new().expect("Could not create test directory"))
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
            directory_listing: false,
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
            directory_listing: false,
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
            directory_listing: false,
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
            directory_listing: false,
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
            directory_listing: false,
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
            directory_listing: false,
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
