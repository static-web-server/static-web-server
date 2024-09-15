#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::{Method, StatusCode};
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
        PathBuf::from("docker/public/")
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
            uri_path: "index.html",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let buf = fs::read(root_dir().join("index.html"))
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
            uri_path: "index.html",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let buf = fs::read(root_dir().join("index.html"))
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
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
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
            redirect_trailing_slash: true,
            compression_static: false,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
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
        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            for uri in ["", "/"] {
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
                        if uri.is_empty() {
                            // it should redirect permanently
                            assert_eq!(res.status(), 308);
                            assert_eq!(res.headers()["location"], "/");

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
        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "/index%2ehtml",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
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
        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            let res1 = match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
            })
            .await
            {
                Ok(result) => match method {
                    // The handle only accepts HEAD or GET request methods
                    Method::GET | Method::HEAD => {
                        let mut res = result.resp;
                        let buf = fs::read(root_dir().join("index.html"))
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

    #[cfg(any(
        feature = "compression",
        feature = "compression-deflate",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd"
    ))]
    #[tokio::test]
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
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                    let res = compression::auto(
                        method,
                        &headers,
                        static_web_server::settings::CompressionLevel::Default,
                        res,
                    )
                    .expect("unexpected bytes error during body compression");

                    let buf = fs::read(root_dir().join("index.html"))
                        .expect("unexpected error during index.html reading");

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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        for method in [Method::HEAD, Method::GET] {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &headers,
                base_path: &root_dir(),
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
        let buf = fs::read(root_dir().join("index.html"))
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
        let buf = fs::read(root_dir().join("index.html"))
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
        let buf = fs::read(root_dir().join("index.html"))
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
                uri_path: "index.html",
                uri_query: None,
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
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
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &[],
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
                #[cfg(feature = "experimental")]
                memory_cache: None,
                #[cfg(feature = "directory-listing")]
                dir_listing: false,
                #[cfg(feature = "directory-listing")]
                dir_listing_order: 6,
                #[cfg(feature = "directory-listing")]
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &["index.html", "index.htm"],
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
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: true,
                index_files: &["index.html", "index.htm"],
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
                redirect_trailing_slash: true,
                compression_static: true,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &["index.html", "index.htm"],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                }
                Err(err) => {
                    match method {
                        // The handle only accepts HEAD or GET request methods
                        Method::GET | Method::HEAD => {
                            panic!("unexpected an error response {}", err)
                        }
                        _ => assert_eq!(err, StatusCode::METHOD_NOT_ALLOWED),
                    }
                }
            }
        }
    }
}
