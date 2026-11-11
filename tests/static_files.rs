#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use http_body_util::BodyExt;
    use static_web_server::exts::http::MethodExt;
    use std::fs;
    use std::path::PathBuf;

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

    /// Root directory for dynamic-compression test fixtures
    /// (files larger than the minimum compression threshold of 200 bytes).
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
            #[cfg(feature = "mem-cache")]
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
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &["index.htm"],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let res = result.resp;

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-length"], buf.len().to_string());
        assert_eq!(res.headers()["accept-ranges"], "bytes");
        assert!(!res.headers()["last-modified"].is_empty());

        let ctype = &res.headers()["content-type"];

        assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

        let body = res
            .into_body()
            .collect()
            .await
            .expect("unexpected bytes error during `body` conversion")
            .to_bytes();

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
            #[cfg(feature = "mem-cache")]
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
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &["index.htm"],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let res = result.resp;

        let buf = fs::read(root_dir().join("index.htm"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-length"], buf.len().to_string());
        assert_eq!(res.headers()["accept-ranges"], "bytes");
        assert!(!res.headers()["last-modified"].is_empty());

        let ctype = &res.headers()["content-type"];

        assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

        let body = res
            .into_body()
            .collect()
            .await
            .expect("unexpected bytes error during `body` conversion")
            .to_bytes();

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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
            #[cfg(feature = "mem-cache")]
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
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &["index.htm"],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let res = result.resp;

        assert_eq!(res.status(), 308);
        assert_eq!(res.headers()["location"], "assets/");

        let body = res
            .into_body()
            .collect()
            .await
            .expect("unexpected bytes error during `body` conversion")
            .to_bytes();

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
            #[cfg(feature = "mem-cache")]
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
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
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
            #[cfg(feature = "mem-cache")]
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
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
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
                    #[cfg(feature = "mem-cache")]
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
                    etag: true,
                    include_hidden: true,
                    follow_symlinks: true,
                    index_files: &[],
                })
                .await
                {
                    Ok(result) => {
                        let res = result.resp;
                        if uri == "/assets" {
                            // it should redirect permanently
                            assert_eq!(res.status(), 308);
                            assert_eq!(res.headers()["location"], "/assets/");

                            let body = res
                                .into_body()
                                .collect()
                                .await
                                .expect("unexpected bytes error during `body` conversion")
                                .to_bytes();

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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 304);
                    assert_eq!(res.headers().get("content-length"), None);
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 412);

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();

                    assert_eq!(body, "");
                }
                Err(_) => {
                    panic!("expected a status 200 but not a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_options_reply_does_not_depend_on_path() {
        // An existing file, a missing path, a directory without trailing slash,
        // a symlink and a hidden file
        let paths = ["index.htm", "missing.html", "assets", "symlink", ".dotfile"];
        let mut replies = Vec::new();
        for uri_path in paths {
            let result = static_files::handle(&HandleOpts {
                method: &Method::OPTIONS,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path,
                uri_query: None,
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: false,
                follow_symlinks: false,
                index_files: &["index.htm"],
            })
            .await;
            let resp = match result {
                Ok(result) => {
                    assert!(result.file_path.is_none(), "{uri_path}");
                    result.resp
                }
                Err(status) => panic!("{uri_path}: unexpected status {status}"),
            };
            replies.push((resp.status(), resp.headers().clone()));
        }

        assert_eq!(replies[0].0, StatusCode::NO_CONTENT);
        for (reply, uri_path) in replies.iter().zip(paths) {
            assert_eq!(reply, &replies[0], "{uri_path}");
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => match method {
                    // The handle only accepts HEAD or GET request methods
                    Method::GET | Method::HEAD => {
                        let res = result.resp;
                        let buf = fs::read(root_dir().join("index.htm"))
                            .expect("unexpected error during index.html reading");
                        let buf = Bytes::from(buf);

                        assert_eq!(res.status(), 200);
                        assert_eq!(res.headers()["content-length"], buf.len().to_string());
                        assert_eq!(res.headers()["accept-ranges"], "bytes");
                        assert!(!res.headers()["last-modified"].is_empty());

                        let ctype = &res.headers()["content-type"];

                        assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

                        let body = res
                            .into_body()
                            .collect()
                            .await
                            .expect("unexpected bytes error during `body` conversion")
                            .to_bytes();

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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 0-0/{}", buf.len())
                    );
                    assert_eq!(res.headers()["content-length"], "1");
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-200/{}", buf.len())
                    );
                    assert_eq!(res.headers()["content-length"], "101");
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert!(res.headers().get("content-length").is_some());
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert_eq!(
                        res.headers()["content-length"],
                        &buf[100..].len().to_string()
                    );
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes {}-{}/{}", buf.len() - 100, buf.len() - 1, buf.len())
                    );
                    assert_eq!(res.headers()["content-length"], "100");
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 416);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes */{}", buf.len())
                    );
                    assert_eq!(res.headers().get("content-length"), None);
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 416);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes */{}", buf.len())
                    );
                    assert!(res.headers().get("content-length").is_none());
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &["index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert!(res.headers().get("content-length").is_some());
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert_eq!(
                        res.headers()["content-length"],
                        format!("{}", buf.len() - 100)
                    );
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 206);
                    assert_eq!(
                        res.headers()["content-range"],
                        format!("bytes 100-{}/{}", buf.len() - 1, buf.len())
                    );
                    assert_eq!(
                        res.headers()["content-length"],
                        format!("{}", buf.len() - 100)
                    );
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    assert_eq!(body, &buf[100..=buf.len() - 1]);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_include_hidden() {
        let root_dir = PathBuf::from("tests/fixtures/public/");
        let headers = HeaderMap::new();

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: ".dotfile",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
                index_files: &[],
            })
            .await
            .expect("unexpected error response on `handle` function");
            let res = result.resp;

            let buf = fs::read(root_dir.join("foo.html"))
                .expect("unexpected error during index.html reading");
            let buf = Bytes::from(buf);

            assert_eq!(res.status(), 200);
            assert_eq!(res.headers()["content-length"], buf.len().to_string());
            assert_eq!(res.headers()["accept-ranges"], "bytes");
            assert!(!res.headers()["last-modified"].is_empty());

            let ctype = &res.headers()["content-type"];

            assert!(ctype == "text/html", "content-type is not html: {ctype:?}",);

            let body = res
                .into_body()
                .collect()
                .await
                .expect("unexpected bytes error during `body` conversion")
                .to_bytes();

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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
                index_files: &["index.htm", "index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-length"], format!("{}", buf.len()));
                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    assert_eq!(body, &buf);
                }
                Err(_) => {
                    panic!("expected a normal response rather than a status error")
                }
            }
        }
    }

    #[tokio::test]
    async fn handle_follow_symlinks() {
        let root_dir = PathBuf::from("tests/fixtures/public/");
        let headers = HeaderMap::new();

        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir,
                uri_path: "/symlink",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: false,
                follow_symlinks: false,
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
                #[cfg(feature = "mem-cache")]
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
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
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
                    #[cfg(feature = "mem-cache")]
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
                    etag: true,
                    include_hidden: false,
                    follow_symlinks: true,
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
                    #[cfg(feature = "mem-cache")]
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
                    etag: true,
                    include_hidden: false,
                    follow_symlinks: true,
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

/// Files served in place of the requested path (index files, `.html`
/// siblings and pre-compressed variants) must pass the same containment and
/// symlink checks as the requested path itself.
#[cfg(all(test, unix))]
mod substitute_file_security_tests {
    use headers::HeaderMap;
    use http::{Method, StatusCode, header::ACCEPT_ENCODING};
    use http_body_util::BodyExt;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    #[cfg(feature = "directory-listing")]
    use static_web_server::directory_listing::DirListFmt;
    use static_web_server::static_files::{self, HandleOpts};

    const OUTSIDE_BODY: &str = "outside-root-marker";
    const INSIDE_BODY: &str = "inside-root-target";

    /// A canonical web root plus symlink targets outside and inside it.
    struct Fixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        outside: PathBuf,
        inside: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("unable to create a temp dir");
            let base = dir
                .path()
                .canonicalize()
                .expect("unable to resolve the temp dir");
            let root = base.join("www");
            std::fs::create_dir_all(root.join("targets")).expect("unable to create the root");
            let outside = base.join("outside-marker");
            std::fs::write(&outside, OUTSIDE_BODY).expect("unable to write the outside file");
            let inside = root.join("targets").join("inside-target");
            std::fs::write(&inside, INSIDE_BODY).expect("unable to write the inside file");
            Self {
                _dir: dir,
                root,
                outside,
                inside,
            }
        }

        fn write(&self, name: &str, body: &str) {
            std::fs::write(self.root.join(name), body).expect("unable to write a fixture file");
        }

        fn link(&self, name: &str, target: &Path) {
            symlink(target, self.root.join(name)).expect("unable to create a symlink");
        }
    }

    /// Requests `uri_path` with `compression-static` on and returns the
    /// status and the body (empty on error).
    async fn get(
        root: &Path,
        uri_path: &str,
        accept_encoding: Option<&str>,
        follow_symlinks: bool,
    ) -> (StatusCode, String) {
        let mut headers = HeaderMap::new();
        if let Some(coding) = accept_encoding {
            headers.insert(ACCEPT_ENCODING, coding.parse().unwrap());
        }
        let base_path = root.to_path_buf();
        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &base_path,
            uri_path,
            uri_query: None,
            #[cfg(feature = "mem-cache")]
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
            etag: false,
            include_hidden: false,
            follow_symlinks,
            index_files: &["index.html"],
        })
        .await;
        match result {
            Ok(res) => {
                let status = res.resp.status();
                let body = res.resp.into_body().collect().await.unwrap().to_bytes();
                (status, String::from_utf8_lossy(&body).into_owned())
            }
            Err(status) => (status, String::new()),
        }
    }

    /// Asserts how a substitute symlink is handled in both symlink modes:
    /// outside the root it's never served, inside the root it follows the
    /// symlink policy.
    async fn assert_symlink_checked(
        uri_path: &str,
        accept_encoding: Option<&str>,
        link_outside: impl Fn(&Fixture),
        link_inside: impl Fn(&Fixture),
    ) {
        for follow_symlinks in [false, true] {
            let fixture = Fixture::new();
            link_outside(&fixture);
            let (status, body) =
                get(&fixture.root, uri_path, accept_encoding, follow_symlinks).await;
            assert_eq!(
                status,
                StatusCode::NOT_FOUND,
                "symlink outside the root must not be served (follow_symlinks={follow_symlinks})"
            );
            assert!(!body.contains(OUTSIDE_BODY));

            let fixture = Fixture::new();
            link_inside(&fixture);
            let (status, body) =
                get(&fixture.root, uri_path, accept_encoding, follow_symlinks).await;
            if follow_symlinks {
                assert_eq!(status, StatusCode::OK);
                assert_eq!(body, INSIDE_BODY);
            } else {
                assert_eq!(
                    status,
                    StatusCode::FORBIDDEN,
                    "symlink inside the root must follow the symlink policy"
                );
            }
        }
    }

    #[tokio::test]
    async fn precompressed_variant_symlink_is_checked() {
        for (ext, coding) in [("gz", "gzip"), ("br", "br"), ("zst", "zstd")] {
            let variant = format!("app.js.{ext}");
            assert_symlink_checked(
                "/app.js",
                Some(coding),
                |f| {
                    f.write("app.js", "original");
                    f.link(&variant, &f.outside);
                },
                |f| {
                    f.write("app.js", "original");
                    f.link(&variant, &f.inside);
                },
            )
            .await;
        }
    }

    #[tokio::test]
    async fn precompressed_variant_regular_file_is_served() {
        let fixture = Fixture::new();
        fixture.write("app.js", "original");
        fixture.write("app.js.gz", "gzip variant");
        let (status, body) = get(&fixture.root, "/app.js", Some("gzip"), false).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "gzip variant");
    }

    #[tokio::test]
    async fn index_file_symlink_is_checked() {
        assert_symlink_checked(
            "/docs/",
            None,
            |f| {
                std::fs::create_dir(f.root.join("docs")).unwrap();
                f.link("docs/index.html", &f.outside);
            },
            |f| {
                std::fs::create_dir(f.root.join("docs")).unwrap();
                f.link("docs/index.html", &f.inside);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn index_file_regular_file_is_served() {
        let fixture = Fixture::new();
        std::fs::create_dir(fixture.root.join("docs")).unwrap();
        fixture.write("docs/index.html", "index");
        let (status, body) = get(&fixture.root, "/docs/", None, false).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "index");
    }

    #[tokio::test]
    async fn html_sibling_of_directory_symlink_is_checked() {
        assert_symlink_checked(
            "/docs/",
            None,
            |f| {
                std::fs::create_dir(f.root.join("docs")).unwrap();
                f.link("docs.html", &f.outside);
            },
            |f| {
                std::fs::create_dir(f.root.join("docs")).unwrap();
                f.link("docs.html", &f.inside);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn html_sibling_of_directory_regular_file_is_served() {
        let fixture = Fixture::new();
        std::fs::create_dir(fixture.root.join("docs")).unwrap();
        fixture.write("docs.html", "sibling");
        let (status, body) = get(&fixture.root, "/docs/", None, false).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "sibling");
    }
}

/// A hidden file or directory must not be served through its 8.3 short name
/// (`GIT~1` for `.git`), which NTFS and FAT volumes resolve on open.
#[cfg(test)]
mod short_name_alias_tests {
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use std::path::Path;

    #[cfg(feature = "directory-listing")]
    use static_web_server::directory_listing::DirListFmt;
    #[cfg(feature = "directory-listing-download")]
    use static_web_server::directory_listing::download::DirDownloadFmt;
    use static_web_server::static_files::{self, HandleOpts};

    /// Requests `uri_path` with directory listing and archive download on,
    /// when built in, and returns the status.
    async fn request(
        root: &Path,
        uri_path: &str,
        uri_query: Option<&str>,
        include_hidden: bool,
    ) -> StatusCode {
        let headers = HeaderMap::new();
        let base_path = root.to_path_buf();
        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &base_path,
            uri_path,
            uri_query,
            #[cfg(feature = "mem-cache")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: true,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[DirDownloadFmt::Targz],
            redirect_trailing_slash: true,
            compression_static: false,
            etag: false,
            include_hidden,
            follow_symlinks: false,
            index_files: &["index.html"],
        })
        .await;
        match result {
            Ok(res) => res.resp.status(),
            Err(status) => status,
        }
    }

    #[tokio::test]
    async fn short_name_check_serves_names_with_tilde() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("a~b")).unwrap();
        std::fs::write(dir.path().join("a~b/file.txt"), "file").unwrap();
        std::fs::write(dir.path().join("report~1.txt"), "report").unwrap();
        std::fs::create_dir(dir.path().join("c~d")).unwrap();
        std::fs::write(dir.path().join("c~d/index.html"), "index").unwrap();

        for uri_path in ["/report~1.txt", "/a~b/file.txt", "/c~d/"] {
            let status = request(dir.path(), uri_path, None, false).await;
            assert_eq!(status, StatusCode::OK, "{uri_path}");
        }
        assert_eq!(
            request(dir.path(), "/c~d", None, false).await,
            StatusCode::PERMANENT_REDIRECT
        );
        #[cfg(feature = "directory-listing")]
        assert_eq!(
            request(dir.path(), "/a~b/", None, false).await,
            StatusCode::OK
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn short_name_check_refuses_names_with_tilde_in_unlistable_dir() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let locked = dir.path().join("locked");
        std::fs::create_dir(&locked).unwrap();
        std::fs::write(locked.join("a~b.txt"), "file").unwrap();
        std::fs::set_permissions(&locked, Permissions::from_mode(0o311)).unwrap();
        let listable = std::fs::read_dir(&locked).is_ok();

        let included = request(dir.path(), "/locked/a~b.txt", None, true).await;
        let excluded = request(dir.path(), "/locked/a~b.txt", None, false).await;
        std::fs::set_permissions(&locked, Permissions::from_mode(0o755)).unwrap();

        if listable {
            eprintln!("skipped: this user can list a directory without read permission");
            return;
        }
        assert_eq!(included, StatusCode::OK);
        assert_eq!(excluded, StatusCode::NOT_FOUND);
    }

    /// Returns the 8.3 short name of `path`, or `None` when the volume has
    /// short names disabled and `cmd` prints the long name back.
    #[cfg(windows)]
    fn short_name(path: &Path) -> Option<String> {
        use std::os::windows::process::CommandExt;

        let out = std::process::Command::new("cmd")
            .arg("/C")
            .raw_arg(format!(
                "for %I in (\"{}\") do @echo %~snxI",
                path.display()
            ))
            .output()
            .expect("unable to run cmd");
        let name = String::from_utf8(out.stdout).expect("cmd printed a non-UTF-8 name");
        let name = name.trim();
        let long_name = path.file_name().unwrap().to_str().unwrap();
        if name == long_name {
            eprintln!("skipped: no 8.3 short names on this volume (cmd printed {name:?})");
            return None;
        }
        assert!(
            out.status.success() && name.contains('~'),
            "unexpected short name {name:?} for {long_name:?}"
        );
        Some(name.to_owned())
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn short_name_alias_of_hidden_path_returns_404() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join(".hidden-dir")).unwrap();
        std::fs::write(root.join(".hidden-dir/file.txt"), "secret").unwrap();
        std::fs::write(root.join(".env"), "secret").unwrap();
        let (Some(dir_alias), Some(file_alias)) = (
            short_name(&root.join(".hidden-dir")),
            short_name(&root.join(".env")),
        ) else {
            return;
        };
        let file_in_dir = format!("/{dir_alias}/file.txt");

        let status = request(root, &file_in_dir, None, true).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{dir_alias} must resolve on this volume"
        );

        let requests = [
            (file_in_dir.clone(), None),
            (format!("/{file_alias}"), None),
            (format!("/{dir_alias}"), None),
            (format!("/{dir_alias}/"), None),
            (format!("/{dir_alias}/"), Some("download=targz")),
        ];
        for (uri_path, uri_query) in requests {
            let status = request(root, &uri_path, uri_query, false).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri_path} {uri_query:?}");
        }
    }
}
