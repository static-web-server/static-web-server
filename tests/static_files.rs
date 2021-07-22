#![deny(warnings)]

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use std::fs;
    use std::path::PathBuf;

    extern crate static_web_server;
    use static_web_server::{compression, static_files};

    fn root_dir() -> PathBuf {
        PathBuf::from("docker/public/")
    }

    #[tokio::test]
    async fn handle_file() {
        let mut res = static_files::handle(
            &Method::GET,
            &HeaderMap::new(),
            root_dir(),
            "index.html",
            false,
        )
        .await
        .expect("unexpected error response on `handle` function");

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-length"], buf.len().to_string());
        assert_eq!(res.headers()["accept-ranges"], "bytes");
        assert!(!res.headers()["last-modified"].is_empty());

        let ctype = &res.headers()["content-type"];

        assert!(
            ctype == "text/html",
            "content-type is not html: {:?}",
            ctype,
        );

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` convertion");

        assert_eq!(body, buf);
    }

    #[tokio::test]
    async fn handle_file_head() {
        let mut res = static_files::handle(
            &Method::HEAD,
            &HeaderMap::new(),
            root_dir(),
            "index.html",
            false,
        )
        .await
        .expect("unexpected error response on `handle` function");

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-length"], buf.len().to_string());
        assert_eq!(res.headers()["accept-ranges"], "bytes");
        assert!(!res.headers()["last-modified"].is_empty());

        let ctype = &res.headers()["content-type"];

        assert!(
            ctype == "text/html",
            "content-type is not html: {:?}",
            ctype,
        );

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` convertion");

        assert_eq!(body, buf);
    }

    #[tokio::test]
    async fn handle_file_not_found() {
        match static_files::handle(
            &Method::GET,
            &HeaderMap::new(),
            root_dir(),
            "xyz.html",
            false,
        )
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

    #[tokio::test]
    async fn handle_file_allowed_disallowed_methods() {
        let methods = [
            Method::CONNECT,
            Method::DELETE,
            Method::GET,
            Method::HEAD,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ];
        for method in methods {
            match static_files::handle(&method, &HeaderMap::new(), root_dir(), "index.html", false)
                .await
            {
                Ok(mut res) => match method {
                    // The handle only accepts HEAD or GET request methods
                    Method::GET | Method::HEAD => {
                        let buf = fs::read(root_dir().join("index.html"))
                            .expect("unexpected error during index.html reading");
                        let buf = Bytes::from(buf);

                        assert_eq!(res.status(), 200);
                        assert_eq!(res.headers()["content-length"], buf.len().to_string());
                        assert_eq!(res.headers()["accept-ranges"], "bytes");
                        assert!(!res.headers()["last-modified"].is_empty());

                        let ctype = &res.headers()["content-type"];

                        assert!(
                            ctype == "text/html",
                            "content-type is not html: {:?}",
                            ctype,
                        );

                        let body = hyper::body::to_bytes(res.body_mut())
                            .await
                            .expect("unexpected bytes error during `body` convertion");

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
    async fn handle_file_compressions() {
        let encodings = ["gzip", "deflate", "br", "xyz"];
        let method = &Method::GET;

        for enc in encodings {
            let mut headers = HeaderMap::new();
            headers.insert(http::header::ACCEPT_ENCODING, enc.parse().unwrap());

            match static_files::handle(method, &headers, root_dir(), "index.html", false).await {
                Ok(res) => {
                    let res = compression::auto(method, &headers, res)
                        .expect("unexpected bytes error during body compression");

                    let buf = fs::read(root_dir().join("index.html"))
                        .expect("unexpected error during index.html reading");

                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["accept-ranges"], "bytes");
                    assert!(!res.headers()["last-modified"].is_empty());

                    match enc {
                        // The handle only accepts `HEAD` or `GET` request methods
                        "gzip" | "deflate" | "br" => {
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

                    assert!(
                        ctype == "text/html",
                        "content-type is not html: {:?}",
                        ctype,
                    );
                }
                Err(_) => {
                    panic!("unexpected status error")
                }
            }
        }
    }
}
