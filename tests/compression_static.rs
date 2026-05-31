#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use http_body_util::BodyExt;
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
        let archive_path = PathBuf::from("tests/fixtures/public/index.htm.br");
        let archive_buf =
            std::fs::read(&archive_path).expect("unexpected error when reading archive file");
        let archive_buf = Bytes::from(archive_buf);

        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression_static: true,
            etag: true,
            index_files: "index.htm, index.html".to_owned(),
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/index.htm".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert!(!headers.contains_key("content-length"));
                assert_eq!(headers["content-encoding"], "br");
                assert_eq!(headers["accept-ranges"], "bytes");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html; charset=utf-8",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();

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
            etag: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/404.html".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert!(!headers.contains_key("content-length"));
                assert_eq!(headers["content-encoding"], "br");
                assert_eq!(headers["accept-ranges"], "bytes");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html; charset=utf-8",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();

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
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression: false,
            compression_static: true,
            etag: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/index.htm".parse().unwrap();
        req.headers_mut()
            .insert(http::header::ACCEPT_ENCODING, "br".parse().unwrap());

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert_eq!(headers["accept-ranges"], "bytes");
                assert_eq!(headers["content-encoding"], "br");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html; charset=utf-8",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[tokio::test]
    #[cfg(feature = "directory-listing")]
    async fn compression_static_index_file() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression: false,
            compression_static: true,
            etag: true,
            directory_listing: true,
            directory_listing_format: DirListFmt::Html,
            include_hidden: true,
            follow_symlinks: true,
            index_files: "index.htm, index.html".to_owned(),
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost".parse().unwrap();
        req.headers_mut()
            .insert(http::header::ACCEPT_ENCODING, "br, gzip".parse().unwrap());

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert_eq!(headers["accept-ranges"], "bytes");
                assert_eq!(headers["content-encoding"], "br");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/html; charset=utf-8",
                    "content-type is not html"
                );
                assert_eq!(headers["vary"], "accept-encoding");
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[tokio::test]
    async fn compression_static_zstd_file_exists() {
        let archive_path = PathBuf::from("tests/fixtures/public/assets/main.css.zst");
        let archive_buf =
            std::fs::read(&archive_path).expect("unexpected error when reading archive file");
        let archive_buf = Bytes::from(archive_buf);

        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression: false,
            compression_static: true,
            etag: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/assets/main.css".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert!(!headers.contains_key("content-length"));
                assert_eq!(headers["content-encoding"], "zstd");
                assert_eq!(headers["accept-ranges"], "bytes");
                assert!(!headers["last-modified"].is_empty());
                assert_eq!(
                    &headers["content-type"], "text/css",
                    "content-type is not css"
                );
                assert_eq!(headers["vary"], "accept-encoding");

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();

                assert_eq!(
                    body, archive_buf,
                    "body and archive_buf are not equal in length"
                );
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    /// When the requested path does not exist on disk and no `.html`-suffixed
    /// sibling exists either, SWS must return 404 without probing for
    /// pre-compressed (`.br`/`.gz`/`.zst`) siblings of the missing path.
    /// See issue #617 for the rationale behind this behavior.
    #[tokio::test]
    async fn compression_static_missing_file_skips_precompressed_probes() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression: false,
            compression_static: true,
            etag: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/this-path-does-not-exist".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 404);
                assert!(!res.headers().contains_key("content-encoding"));
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    /// When the requested path does not exist but its `.html`-suffixed
    /// sibling does, SWS must resolve the suffixed path and, if a
    /// pre-compressed sibling of *that* `.html` file exists, serve it.
    ///
    /// Before the issue #617 fix the pre-compressed `.html` sibling was
    /// ignored on the html-suffix fallback path.
    #[tokio::test]
    async fn compression_static_html_suffix_serves_precompressed_variant() {
        let archive_path = PathBuf::from("tests/fixtures/public/404.html.br");
        let archive_buf =
            std::fs::read(&archive_path).expect("unexpected error when reading archive file");
        let archive_buf = Bytes::from(archive_buf);

        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression: false,
            compression_static: true,
            etag: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        // Request `/404` (no extension): SWS appends `.html`, finds
        // `404.html`, and should then serve the pre-compressed
        // `404.html.br` sibling.
        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/404".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                let headers = res.headers();

                assert_eq!(res.status(), 200);
                assert_eq!(headers["content-encoding"], "br");
                assert_eq!(headers["vary"], "accept-encoding");
                assert_eq!(
                    &headers["content-type"], "text/html; charset=utf-8",
                    "content-type is not html"
                );

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();

                assert_eq!(body, archive_buf, "body must equal the .html.br archive");
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    /// When a directory has no index file at all, SWS must fall back to
    /// the directory listing (or 404 when listing is disabled) without
    /// probing for pre-compressed siblings of the non-existent index.
    ///
    /// Regression for issue #617: previously the in-loop and post-loop
    /// pre-compressed probes ran even when the index file did not exist,
    /// wasting `stat(2)` syscalls per configured encoding.
    #[tokio::test]
    async fn compression_static_missing_index_skips_precompressed_probes() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            #[cfg(any(
                feature = "compression",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd",
                feature = "compression-deflate"
            ))]
            compression: false,
            compression_static: true,
            etag: true,
            // Use only an index name that does not exist in `assets/` so
            // SWS exercises the no-index-found branch.
            index_files: "missing-index.html".to_owned(),
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/assets/".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                // Directory listing is disabled in the default fixture, so
                // the response must be a plain 404 with no content-encoding
                // header from a stray pre-compressed match.
                assert_eq!(res.status(), 404);
                assert!(!res.headers().contains_key("content-encoding"));
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }
}
