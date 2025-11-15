#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use headers::HeaderValue;
    use hyper::Request;
    use std::net::SocketAddr;

    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    #[tokio::test]
    async fn markdown_disabled_returns_html() {
        let opts = fixture_settings("toml/markdown_disabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                // Should return HTML because markdown is disabled
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("Article HTML"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_get_returns_markdown() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                // Should return markdown with correct Content-Type
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/markdown; charset=utf-8"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("# Article Markdown"));
                assert!(body_str.contains("This is the markdown source version"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_head_returns_markdown() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::HEAD;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                // Should have markdown Content-Type
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/markdown; charset=utf-8"))
                );
                // Should have Content-Length header
                assert!(res.headers().get("content-length").is_some());
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_without_accept_header_returns_html() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        // No Accept header - should return HTML

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("Article HTML"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_wildcard_accept_returns_html() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        req.headers_mut()
            .insert(hyper::header::ACCEPT, HeaderValue::from_static("text/*"));

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                // Wildcard should NOT trigger markdown negotiation
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("Article HTML"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_direct_md_file() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/doc".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/markdown; charset=utf-8"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("# Documentation"));
                assert!(body_str.contains("Direct markdown file"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_directory_index() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/markdown; charset=utf-8"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("# Index Markdown"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_post_not_handled() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::POST;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                // POST should not be handled (405 Method Not Allowed)
                assert_eq!(res.status(), 405);
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_get_html_with_markdown_header() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/test.html".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                // Should return HTML content-type since no markdown variant exists
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("Test Page"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_head_html_with_markdown_header() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::HEAD;
        *req.uri_mut() = "http://localhost/test.html".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                // Should return HTML content-type since no markdown variant exists
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );
                // Should have Content-Length header
                assert!(res.headers().get("content-length").is_some());
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_get_html_without_markdown_header() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article.html".parse().unwrap();
        // No Accept header - should return HTML with HTML content-type

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );

                let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                assert!(body_str.contains("Article HTML"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_head_html_without_markdown_header() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::HEAD;
        *req.uri_mut() = "http://localhost/article.html".parse().unwrap();
        // No Accept header - should return HTML with HTML content-type

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );
                // Should have Content-Length header
                assert!(res.headers().get("content-length").is_some());
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_404_when_no_variant() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/nonexistent".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                // Should return 404 when no markdown variant exists
                assert_eq!(res.status(), 404);
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }
}
