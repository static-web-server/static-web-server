#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(any(
    feature = "compression",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd"
))]
#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use hyper::Request;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    #[cfg(feature = "directory-listing")]
    use static_web_server::directory_listing::DirListFmt;
    use static_web_server::{
        settings::cli::General,
        testing::fixtures::{
            REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
        },
    };

    #[tokio::test]
    async fn compression_static_file_exists() {
        let archive_path = PathBuf::from("tests/fixtures/public/index.html.gz");
        let archive_buf =
            std::fs::read(&archive_path).expect("unexpected error when reading archive file");
        let archive_buf = Bytes::from(archive_buf);

        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression_static: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/index.html".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(mut res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert!(!headers.contains_key("content-length"));
                assert_eq!(headers["content-encoding"], "gzip");
                assert_eq!(headers["accept-ranges"], "bytes");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");

                let body = hyper::body::to_bytes(res.body_mut())
                    .await
                    .expect("unexpected bytes error during `body` conversion");

                assert_eq!(
                    body, archive_buf,
                    "body and archive_buf are not equal in length"
                );
            }
            Err(err) => panic!("unexpected error: {err}"),
        };
    }

    #[tokio::test]
    async fn compression_static_suboptimal_file_exists() {
        let archive_path = PathBuf::from("tests/fixtures/public/404.html.br");
        let archive_buf =
            std::fs::read(&archive_path).expect("unexpected error when reading archive file");
        let archive_buf = Bytes::from(archive_buf);

        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression_static: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/404.html".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(mut res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert!(!headers.contains_key("content-length"));
                assert_eq!(headers["content-encoding"], "br");
                assert_eq!(headers["accept-ranges"], "bytes");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");

                let body = hyper::body::to_bytes(res.body_mut())
                    .await
                    .expect("unexpected bytes error during `body` conversion");

                assert_eq!(
                    body, archive_buf,
                    "body and archive_buf are not equal in length"
                );
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[tokio::test]
    async fn compression_static_file_does_not_exist() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression: false,
            compression_static: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/index.htm".parse().unwrap();
        req.headers_mut()
            .insert(http::header::ACCEPT_ENCODING, "br".parse().unwrap());

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert!(headers.contains_key("content-length"));
                assert_eq!(headers["accept-ranges"], "bytes");
                assert!(!headers.contains_key("content-encoding"));
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[cfg(feature = "directory-listing")]
    #[tokio::test]
    async fn compression_static_index_file() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression: false,
            compression_static: true,
            directory_listing: true,
            directory_listing_format: DirListFmt::Html,
            ignore_hidden_files: false,
            disable_symlinks: false,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost".parse().unwrap();
        req.headers_mut()
            .insert(http::header::ACCEPT_ENCODING, "zstd, gzip".parse().unwrap());

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert_eq!(headers["accept-ranges"], "bytes");
                assert_eq!(headers["content-encoding"], "gzip");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[tokio::test]
    async fn compression_static_zstd_file_exists() {
        let archive_path = PathBuf::from("tests/fixtures/public/main.js.zst");
        let archive_buf =
            std::fs::read(&archive_path).expect("unexpected error when reading archive file");
        let archive_buf = Bytes::from(archive_buf);

        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression: false,
            compression_static: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/main.js".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(mut res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert!(!headers.contains_key("content-length"));
                assert_eq!(headers["content-encoding"], "zstd");
                assert_eq!(headers["accept-ranges"], "bytes");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/javascript",
                    "content-type is not javascript"
                );
                assert_eq!(headers["vary"], "accept-encoding");

                let body = hyper::body::to_bytes(res.body_mut())
                    .await
                    .expect("unexpected bytes error during `body` conversion");

                assert_eq!(
                    body, archive_buf,
                    "body and archive_buf are not equal in length"
                );
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }
}
