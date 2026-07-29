#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use static_web_server::http_ext::MethodExt;
    use static_web_server::static_files::StaticFileResponse;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    #[cfg(any(
        feature = "compression",
        feature = "compression-deflate",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd"
    ))]
    use static_web_server::compression;

    #[cfg(feature = "directory-listing")]
    use static_web_server::directory_listing::DirListFmt;
    use static_web_server::static_files::{self, HandleOpts};

    fn root_dir() -> PathBuf {
        PathBuf::from("tests/fixtures/public/")
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let path = std::env::temp_dir().join(format!(
                "sws-static-files-{tag}-{}-{nanos}-{seq}",
                std::process::id(),
            ));
            fs::create_dir_all(&path).expect("unexpected error creating temporary directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[cfg(any(
        feature = "compression",
        feature = "compression-deflate",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd"
    ))]
    fn comp_root_dir() -> PathBuf {
        PathBuf::from("tests/fixtures/compression/")
    }

    const METHODS: [Method; 8] = [
        Method::CONNECT,
        Method::DELETE,
        Method::GET,
        Method::HEAD,
        Method::PATCH,
        Method::POST,
        Method::PUT,
        Method::TRACE,
    ];

    #[tokio::test]
    async fn handle_file() {
        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: &root_dir(),
            uri_path: "index.htm",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &["index.htm"],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-length"], buf.len().to_string());
        assert_eq!(res.headers()["accept-ranges"], "bytes");
        assert!(!res.headers()["last-modified"].is_empty());

        let ctype = &res.headers()["content-type"];

        assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(body, buf);
    }

    #[tokio::test]
    async fn handle_file_head() {
        let result = static_files::handle(&HandleOpts {
            method: &Method::HEAD,
            headers: &HeaderMap::new(),
            base_path: &root_dir(),
            uri_path: "index.htm",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &["index.htm"],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-length"], buf.len().to_string());
        assert_eq!(res.headers()["accept-ranges"], "bytes");
        assert!(!res.headers()["last-modified"].is_empty());

        let ctype = &res.headers()["content-type"];

        assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(body, buf);
    }

    #[tokio::test]
    async fn handle_file_not_found() {
        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "xyz.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(_) => {
                    panic!("expected a status error 404 but not status 200")
                }
                Err(status) => {
                    assert_eq!(status, StatusCode::NOT_FOUND);
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_trailing_slash_redirection() {
        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: &root_dir(),
            uri_path: "assets",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 0,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &["index.htm"],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        assert_eq!(res.status(), 308);
        assert_eq!(res.headers()["location"], "assets/");

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(body, Bytes::new());
    }

    #[tokio::test]
    async fn handle_trailing_slash_redirection_subdir() {
        match static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: &root_dir(),
            uri_path: "assets",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 0,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &["index.htm"],
        })
        .await
        {
            Ok(result) => {
                let res = result.resp;
                assert_eq!(res.status(), 308);
                assert_eq!(res.headers()["location"], "assets/");
            }
            Err(status) => {
                panic!("expected a status 308 but not a status {status}")
            }
        }
    }

    #[tokio::test]
    async fn handle_disabled_trailing_slash_redirection_subdir() {
        match static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: &root_dir(),
            uri_path: "assets",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 0,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: false,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        {
            Ok(result) => {
                let res = result.resp;
                assert_eq!(res.status(), 200);
            }
            Err(status) => {
                panic!("expected a status 200 but not a status {status}")
            }
        }
    }

    #[tokio::test]
    async fn handle_append_index_on_dir() {
        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            for uri in ["/assets", "/assets/"] {
                match static_files::handle(&HandleOpts {
                    method: &method,
                    headers: &HeaderMap::new(),
                    base_path: &root_dir(),
                    uri_path: uri,
                    uri_query: None,
                    #[cfg(feature = "experimental")]
                    memory_cache: None,
                    #[cfg(feature = "directory-listing")]
                    dir_listing: false,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_order: 6,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_format: &DirListFmt::Html,
                    #[cfg(feature = "directory-listing-download")]
                    dir_listing_download: &[],
                    redirect_trailing_slash: true,
                    compression_static: false,
                    ignore_hidden_files: false,
                    disable_symlinks: false,
                    index_files: &[],
                })
                .await
                {
                    Ok(result) => {
                        let mut res = result.resp;
                        if uri == "/assets" {
                            // it should redirect permanently
                            assert_eq!(res.status(), 308);
                            assert_eq!(res.headers()["location"], "/assets/");

                            let body = hyper::body::to_bytes(res.body_mut())
                                .await
                                .expect("unexpected bytes error during `body` conversion");

                            assert_eq!(body, Bytes::new());
                        } else {
                            // otherwise it should response with ok
                            assert_eq!(res.status(), 200);
                            assert_eq!(res.headers()["content-length"], buf.len().to_string());
                        }
                    }
                    Err(_) => {
                        panic!("expected a status 200 but not a status error")
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_file_encoded() {
        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "/assets/index%2ehtml",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-length"], buf.len().to_string());
                }
                Err(_) => {
                    panic!("expected a status 200 but not a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_bad_encoded_path() {
        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "/%2E%2e.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(_) => {
                    panic!("expected a status 200 but not a status error")
                }
                Err(status) => {
                    assert_eq!(status, 404);
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_not_modified() {
        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            let res1 = match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-length"], buf.len().to_string());
                    res
                }
                Err(_) => {
                    panic!("expected a status 200 but not a status error")
                }
            };

            // if-modified-since
            let mut headers = HeaderMap::new();
            headers.insert(
                "if-modified-since",
                res1.headers()["last-modified"].to_owned(),
            );

            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 304);
                    assert_eq!(res.headers().get("content-length"), None);
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, "");
                }
                Err(_) => {
                    panic!("expected a status 304 but not a status error")
                }
            }

            // clearly too old
            let mut headers = HeaderMap::new();
            headers.insert(
                "if-modified-since",
                "Mon, 18 Nov 1974 00:00:00 GMT".parse().unwrap(),
            );

            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 200);
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, buf);
                    assert_eq!(res1.headers()["content-length"], buf.len().to_string());
                }
                Err(_) => {
                    panic!("expected a status 200 but not a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_precondition() {
        for method in [Method::HEAD, Method::GET] {
            let res1 = match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    res
                }
                Err(_) => {
                    panic!("expected a status 200 but not a status error")
                }
            };

            // if-unmodified-since
            let mut headers = HeaderMap::new();
            headers.insert(
                "if-unmodified-since",
                res1.headers()["last-modified"].to_owned(),
            );

            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                }
                Err(_) => {
                    panic!("expected a status 200 but not a status error")
                }
            }

            // clearly too old
            let mut headers = HeaderMap::new();
            headers.insert(
                "if-unmodified-since",
                "Mon, 18 Nov 1974 00:00:00 GMT".parse().unwrap(),
            );

            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 412);

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");

                    assert_eq!(body, "");
                }
                Err(_) => {
                    panic!("expected a status 200 but not a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_file_allowed_disallowed_methods() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => match method {
                    // The handle only accepts HEAD or GET request methods
                    Method::GET | Method::HEAD => {
                        let mut res = result.resp;
                        let buf = fs::read(root_dir().join("index.htm"))
                            .expect("unexpected error during index.html reading");
                        let buf = Bytes::from(buf);

                        assert_eq!(res.status(), 200);
                        assert_eq!(res.headers()["content-length"], buf.len().to_string());
                        assert_eq!(res.headers()["accept-ranges"], "bytes");
                        assert!(!res.headers()["last-modified"].is_empty());

                        let ctype = &res.headers()["content-type"];

                        assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

                        let body = hyper::body::to_bytes(res.body_mut())
                            .await
                            .expect("unexpected bytes error during `body` conversion");

                        assert_eq!(body, buf);
                    }
                    _ => {
                        panic!("unexpected response for method {}", method.as_str())
                    }
                },
                Err(status) => {
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    #[cfg(any(
        feature = "compression",
        feature = "compression-deflate",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd"
    ))]
    async fn handle_file_compressions() {
        let encodings = [
            #[cfg(any(feature = "compression", feature = "compression-gzip"))]
            "gzip",
            #[cfg(any(feature = "compression", feature = "compression-deflate"))]
            "deflate",
            #[cfg(any(feature = "compression", feature = "compression-brotli"))]
            "br",
            #[cfg(any(feature = "compression", feature = "compression-zstd"))]
            "zstd",
            "xyz",
        ];
        let method = &Method::GET;

        for enc in encodings {
            let mut headers = HeaderMap::new();
            headers.insert(
                http::header::ACCEPT_ENCODING,
                format!("identity, {enc}").parse().unwrap(),
            );

            match static_files::handle(&HandleOpts {
                method,
                headers: &headers,
                base_path: &comp_root_dir(),
                uri_path: "large-test.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["large-test.html"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    let res = compression::auto(
                        method,
                        &headers,
                        static_web_server::settings::CompressionLevel::Default,
                        res,
                    )
                    .expect("unexpected bytes error during body compression");

                    let buf = fs::read(comp_root_dir().join("large-test.html"))
                        .expect("unexpected error during large-test.html reading");

                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["accept-ranges"], "bytes");
                    assert!(!res.headers()["last-modified"].is_empty());

                    match enc {
                        // The handle only accepts `HEAD` or `GET` request methods
                        "gzip" | "deflate" | "br" | "zstd" => {
                            assert!(res.headers().get("content-length").is_none());
                            assert_eq!(res.headers()["content-encoding"], enc);
                        }
                        _ => {
                            // otherwise the compression doesn't happen because unsupported `accept-encoding`
                            assert_eq!(res.headers()["content-length"], buf.len().to_string());
                            assert!(res.headers().get("content-encoding").is_none());
                        }
                    };

                    let ctype = &res.headers()["content-type"];

                    assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);
                }
                Err(_) => {
                    panic!("unexpected status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_single() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=0-0".parse().unwrap());

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 0-0/{}", buf.len())
                    );
                    assert_eq!(res.headers()["content-length"], "1");
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, &buf[..=0]);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_multiple() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=100-200".parse().unwrap());

        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "assets/index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-200/{}", buf.len())
                    );
                    assert_eq!(res.headers()["content-length"], "101");
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, &buf[100..=200]);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_out_of_range() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=100-100000".parse().unwrap());

        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "assets/index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert!(res.headers().get("content-length").is_some());
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert!(body.len() > 400);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_if_range_too_old() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=100-200".parse().unwrap());
        headers.insert("if-range", "Mon, 18 Nov 1974 00:00:00 GMT".parse().unwrap());

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-length"], buf.len().to_string());
                    assert_eq!(res.headers().get("content-range"), None);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_suffix() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=100-".parse().unwrap());

        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "assets/index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert_eq!(
                        res.headers()["content-length"],
                        &buf[100..].len().to_string()
                    );
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, &buf[100..]);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_suffix_2() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=-100".parse().unwrap());

        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "assets/index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes {}-{}/{}", buf.len() - 100, buf.len() - 1, buf.len())
                    );
                    assert_eq!(res.headers()["content-length"], "100");
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, &buf[buf.len() - 100..]);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_bad() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=100-10".parse().unwrap());

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 416);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes */{}", buf.len())
                    );
                    assert_eq!(res.headers().get("content-length"), None);
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, "");
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_bad_non_numeric() {
        let mut headers = HeaderMap::new();
        headers.insert("range", "bytes=xyx-abc".parse().unwrap());

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 416);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes */{}", buf.len())
                    );
                    assert!(res.headers().get("content-length").is_none());
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert!(body.is_empty());
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_bad_2() {
        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        let mut headers = HeaderMap::new();
        headers.insert(
            "range",
            format!("bytes=-{}", buf.len() + 1).parse().unwrap(),
        );

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "assets/index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert!(res.headers().get("content-length").is_some());
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert!(body.len() > 500);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_bad_3() {
        let mut headers = HeaderMap::new();
        // Range::Unbounded for beginning and end
        headers.insert("range", "bytes=".parse().unwrap());

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.htm",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 416);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_exclude_file_size() {
        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        let mut headers = HeaderMap::new();
        // range including end of file (non-inclusive result)
        headers.insert("range", format!("bytes=100-{}", buf.len()).parse().unwrap());

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "assets/index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert_eq!(
                        res.headers()["content-length"],
                        format!("{}", buf.len() - 100)
                    );
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, &buf[100..=buf.len() - 1]);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_byte_ranges_exclude_file_size_2() {
        let buf = fs::read(root_dir().join("assets/index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        let mut headers = HeaderMap::new();
        // range with 1 byte to end yields same result as above. (inclusive result)
        headers.insert(
            "range",
            format!("bytes=100-{}", buf.len() - 1).parse().unwrap(),
        );

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "assets/index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert_eq!(
                        res.headers()["content-length"],
                        format!("{}", buf.len() - 100)
                    );
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, &buf[100..=buf.len() - 1]);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_ignore_hidden_files() {
        let root_dir = PathBuf::from("tests/fixtures/public/");
        let headers = HeaderMap::new();

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: ".dotfile",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(_) => {
                    panic!("expected a status error 404 but not status 200")
                }
                Err(status) => {
                    assert_eq!(status, StatusCode::NOT_FOUND);
                }
            }
        }
    }

    #[tokio::test]
    async fn hidden_base_path_not_ignored() {
        let root_dir = PathBuf::from("tests/fixtures/.hidden-root");
        let headers = HeaderMap::new();

        for method in [Method::HEAD, Method::GET] {
            let result = static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: "foo.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            .expect("unexpected error response on `handle` function");
            let mut res = result.resp;

            let buf = fs::read(root_dir.join("foo.html"))
                .expect("unexpected error during index.html reading");
            let buf = Bytes::from(buf);

            assert_eq!(res.status(), 200);
            assert_eq!(res.headers()["content-length"], buf.len().to_string());
            assert_eq!(res.headers()["accept-ranges"], "bytes");
            assert!(!res.headers()["last-modified"].is_empty());

            let ctype = &res.headers()["content-type"];

            assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

            let body = hyper::body::to_bytes(res.body_mut())
                .await
                .expect("unexpected bytes error during `body` conversion");

            assert_eq!(body, buf);
        }
    }

    #[tokio::test]
    async fn hidden_file_in_hidden_base_path_ignored() {
        let root_dir = PathBuf::from("tests/fixtures/.hidden-root");
        let headers = HeaderMap::new();

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: ".hidden-file.txt",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(_) => {
                    panic!("expected a status error 404 but not status 200")
                }
                Err(status) => {
                    assert_eq!(status, StatusCode::NOT_FOUND);
                }
            }
        }
    }

    async fn request_precompressed_file(
        root_dir: &PathBuf,
        uri_path: &str,
        encoding: &str,
        disable_symlinks: bool,
    ) -> Result<StaticFileResponse, StatusCode> {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            encoding.parse().expect("unexpected invalid encoding"),
        );

        static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: root_dir,
            uri_path,
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: true,
            ignore_hidden_files: false,
            disable_symlinks,
            index_files: &[],
        })
        .await
    }

    #[tokio::test]
    async fn handle_precompressed_response_uses_selected_file_metadata() {
        let temp_dir = TempDir::new("precompressed-metadata");
        let root_dir = temp_dir.path().join("public");
        fs::create_dir(&root_dir).expect("unexpected error creating web root");
        fs::write(root_dir.join("app.js"), b"x").expect("unexpected error writing original file");
        let precompressed_body = b"pre-compressed-response-body";
        fs::write(root_dir.join("app.js.gz"), precompressed_body)
            .expect("unexpected error writing pre-compressed file");

        let mut response = request_precompressed_file(&root_dir, "app.js", "gzip", false)
            .await
            .expect("expected pre-compressed response")
            .resp;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[http::header::CONTENT_ENCODING], "gzip");
        let body = hyper::body::to_bytes(response.body_mut())
            .await
            .expect("unexpected bytes error during body conversion");
        assert_eq!(body.as_ref(), precompressed_body);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn handle_precompressed_symlinks_outside_root_are_rejected() {
        let temp_dir = TempDir::new("precompressed-outside");
        let root_dir = temp_dir.path().join("public");
        fs::create_dir(&root_dir).expect("unexpected error creating web root");
        let outside_marker = temp_dir.path().join("outside-marker");
        fs::write(&outside_marker, "outside").expect("unexpected error writing outside marker");

        for (extension, encoding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            for disable_symlinks in [false, true] {
                let asset_name = format!("outside-{extension}-{disable_symlinks}.js");
                fs::write(root_dir.join(&asset_name), "source")
                    .expect("unexpected error writing asset");
                symlink(
                    &outside_marker,
                    root_dir.join(format!("{asset_name}.{extension}")),
                )
                .expect("unexpected error creating precompressed symlink");

                match request_precompressed_file(&root_dir, &asset_name, encoding, disable_symlinks)
                    .await
                {
                    Ok(result) => panic!(
                        "expected {encoding} symlink outside root to be rejected, got {}",
                        result.resp.status()
                    ),
                    Err(status) => assert_eq!(
                        status,
                        StatusCode::NOT_FOUND,
                        "unexpected status for {encoding} with disable_symlinks={disable_symlinks}"
                    ),
                }
            }
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn handle_precompressed_symlinks_inside_root_are_rejected_when_disabled() {
        let temp_dir = TempDir::new("precompressed-inside-blocked");
        let root_dir = temp_dir.path().join("public");
        fs::create_dir(&root_dir).expect("unexpected error creating web root");
        let inside_marker = root_dir.join("inside-marker");
        fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");

        for (extension, encoding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            let asset_name = format!("inside-blocked-{extension}.js");
            fs::write(root_dir.join(&asset_name), "source")
                .expect("unexpected error writing asset");
            symlink(
                &inside_marker,
                root_dir.join(format!("{asset_name}.{extension}")),
            )
            .expect("unexpected error creating precompressed symlink");

            match request_precompressed_file(&root_dir, &asset_name, encoding, true).await {
                Ok(result) => panic!(
                    "expected {encoding} symlink to be rejected, got {}",
                    result.resp.status()
                ),
                Err(status) => assert_eq!(
                    status,
                    StatusCode::FORBIDDEN,
                    "unexpected status for {encoding}"
                ),
            }
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn handle_precompressed_symlinks_inside_root_are_served_when_enabled() {
        let temp_dir = TempDir::new("precompressed-inside-allowed");
        let root_dir = temp_dir.path().join("public");
        fs::create_dir(&root_dir).expect("unexpected error creating web root");
        let inside_marker = root_dir.join("inside-marker");
        fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");

        for (extension, encoding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            let asset_name = format!("inside-allowed-{extension}.js");
            fs::write(root_dir.join(&asset_name), "source")
                .expect("unexpected error writing asset");
            symlink(
                &inside_marker,
                root_dir.join(format!("{asset_name}.{extension}")),
            )
            .expect("unexpected error creating precompressed symlink");

            let result = request_precompressed_file(&root_dir, &asset_name, encoding, false)
                .await
                .expect("expected contained precompressed symlink to be served");
            let mut response = result.resp;

            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()[http::header::CONTENT_ENCODING], encoding);
            let body = hyper::body::to_bytes(response.body_mut())
                .await
                .expect("unexpected bytes error during body conversion");
            assert_eq!(body.as_ref(), b"inside");
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn handle_precompressed_symlinks_retargeted_outside_root_are_rejected() {
        let temp_dir = TempDir::new("precompressed-retargeted");
        let root_dir = temp_dir.path().join("public");
        fs::create_dir(&root_dir).expect("unexpected error creating web root");
        let inside_marker = root_dir.join("inside-marker");
        fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");
        let outside_marker = temp_dir.path().join("outside-marker");
        fs::write(&outside_marker, "beyond").expect("unexpected error writing outside marker");

        for (extension, encoding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            let asset_name = format!("retargeted-{extension}.js");
            fs::write(root_dir.join(&asset_name), "source")
                .expect("unexpected error writing asset");
            let precompressed_path = root_dir.join(format!("{asset_name}.{extension}"));

            symlink(&inside_marker, &precompressed_path)
                .expect("unexpected error creating contained precompressed symlink");
            request_precompressed_file(&root_dir, &asset_name, encoding, false)
                .await
                .expect("expected contained precompressed symlink to be served");

            fs::remove_file(&precompressed_path)
                .expect("unexpected error removing contained precompressed symlink");
            symlink(&outside_marker, &precompressed_path)
                .expect("unexpected error retargeting precompressed symlink");

            match request_precompressed_file(&root_dir, &asset_name, encoding, false).await {
                Ok(result) => panic!(
                    "expected retargeted {encoding} symlink to be rejected, got {}",
                    result.resp.status()
                ),
                Err(status) => assert_eq!(status, StatusCode::NOT_FOUND),
            }
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn handle_requested_symlink_retargeted_outside_root_is_rejected() {
        let temp_dir = TempDir::new("requested-retargeted");
        let root_dir = temp_dir.path().join("public");
        fs::create_dir(&root_dir).expect("unexpected error creating web root");
        let inside_marker = root_dir.join("inside-marker");
        fs::write(&inside_marker, "inside").expect("unexpected error writing inside marker");
        let outside_marker = temp_dir.path().join("outside-marker");
        fs::write(&outside_marker, "beyond").expect("unexpected error writing outside marker");
        let requested_path = root_dir.join("app.js");

        symlink(&inside_marker, &requested_path)
            .expect("unexpected error creating contained requested symlink");
        request_precompressed_file(&root_dir, "app.js", "identity", false)
            .await
            .expect("expected contained requested symlink to be served");

        fs::remove_file(&requested_path)
            .expect("unexpected error removing contained requested symlink");
        symlink(&outside_marker, &requested_path)
            .expect("unexpected error retargeting requested symlink");

        match request_precompressed_file(&root_dir, "app.js", "identity", false).await {
            Ok(result) => panic!(
                "expected retargeted requested symlink to be rejected, got {}",
                result.resp.status()
            ),
            Err(status) => assert_eq!(status, StatusCode::NOT_FOUND),
        }
    }

    #[tokio::test]
    async fn handle_multiple_index_files() {
        let root_dir = PathBuf::from("tests/fixtures/public/");
        let headers = HeaderMap::new();

        let buf = fs::read(root_dir.join("index.htm"))
            .expect("unexpected error during index.htm reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &["index.htm", "index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-length"], format!("{}", buf.len()));
                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    assert_eq!(body, &buf);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_disable_symlinks() {
        let root_dir = PathBuf::from("tests/fixtures/public/");
        let headers = HeaderMap::new();

        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: "/symlink",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: true,
                index_files: &["index.htm", "index.htm"],
            })
            .await
            {
                Ok(_) => panic!("unexpected successful response rather than an error"),
                Err(err) => {
                    match method {
                        // The handle only accepts HEAD or GET request methods
                        Method::GET | Method::HEAD => assert_eq!(err, StatusCode::FORBIDDEN),
                        _ => assert_eq!(err, StatusCode::METHOD_NOT_ALLOWED),
                    }
                }
            }
        }

        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: "/symlink/spécial file.txt~",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &["index.htm", "index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                }
                Err(status) => match method {
                    Method::GET | Method::HEAD => {
                        panic!("unexpected error response with status {status}")
                    }
                    _ => assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED),
                },
            }
        }
    }

    #[tokio::test]
    async fn handle_symlinks_paths() {
        let root_dir_rel = PathBuf::from("tests/fixtures/public/");
        let root_dir_abs = root_dir_rel.canonicalize().unwrap();
        let headers = HeaderMap::new();

        for root_dir in [root_dir_rel, root_dir_abs] {
            for method in METHODS {
                match static_files::handle(&HandleOpts {
                    method: &method,
                    headers: &headers,
                    base_path: &root_dir,
                    uri_path: "/readme.md",
                    uri_query: None,
                    #[cfg(feature = "experimental")]
                    memory_cache: None,
                    #[cfg(feature = "directory-listing")]
                    dir_listing: false,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_order: 6,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_format: &DirListFmt::Html,
                    #[cfg(feature = "directory-listing-download")]
                    dir_listing_download: &[],
                    redirect_trailing_slash: true,
                    compression_static: true,
                    ignore_hidden_files: true,
                    disable_symlinks: false,
                    index_files: &["index.htm", "index.htm"],
                })
                .await
                {
                    Ok(_) => {
                        panic!("unexpected successful response")
                    }
                    Err(status) => {
                        if method.is_allowed() {
                            assert_eq!(status, StatusCode::NOT_FOUND)
                        } else {
                            assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED)
                        }
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_symlinks_skip_broken_path() {
        let root_dir_rel = PathBuf::from("tests/fixtures/symlink/");
        let root_dir_abs = root_dir_rel.canonicalize().unwrap();
        let headers = HeaderMap::new();

        for root_dir in [root_dir_rel, root_dir_abs] {
            for method in METHODS {
                match static_files::handle(&HandleOpts {
                    method: &method,
                    headers: &headers,
                    base_path: &root_dir,
                    uri_path: "/unknown.md",
                    uri_query: None,
                    #[cfg(feature = "experimental")]
                    memory_cache: None,
                    #[cfg(feature = "directory-listing")]
                    dir_listing: false,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_order: 6,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_format: &DirListFmt::Html,
                    #[cfg(feature = "directory-listing-download")]
                    dir_listing_download: &[],
                    redirect_trailing_slash: true,
                    compression_static: true,
                    ignore_hidden_files: true,
                    disable_symlinks: false,
                    index_files: &["index.htm", "index.htm"],
                })
                .await
                {
                    Ok(_) => {
                        panic!("unexpected successful response")
                    }
                    Err(status) => {
                        if method.is_allowed() {
                            assert_eq!(status, StatusCode::NOT_FOUND)
                        } else {
                            assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED)
                        }
                    }
                }
            }
        }
    }
}
