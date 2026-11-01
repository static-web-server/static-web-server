#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use headers::HeaderValue;
    use http_body_util::BodyExt;
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

        let mut req = Request::new(());
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
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
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

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
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

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        // No Accept header - should return HTML

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
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
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
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

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
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

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
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

        let mut req = Request::new(());
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
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
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
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
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

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article.html".parse().unwrap();
        // No Accept header - should return HTML with HTML content-type

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );

                let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
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

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::HEAD;
        *req.uri_mut() = "http://localhost/article.html".parse().unwrap();
        // No Accept header - should return HTML with HTML content-type

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
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
    async fn markdown_enabled_traversal_returns_404() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        // Attempt to escape the document root and probe for a file outside it.
        *req.uri_mut() = "http://localhost/../README.md".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                // The markdown pre-processor must sanitize the path; the
                // request falls through to the static file handler and
                // returns 404.
                assert_eq!(res.status(), 404);
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_percent_encoded_traversal_returns_404() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/%2e%2e/%2e%2e/CHANGELOG.md"
            .parse()
            .unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 404);
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_hidden_file_returns_404() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/.dotfile".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 404);
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn markdown_enabled_vary_accept_on_both_representations() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        // The same URI must vary on `Accept` whether it resolves to markdown or HTML
        for accept in [Some("text/markdown"), Some("text/html"), None] {
            let mut req = Request::new(());
            *req.method_mut() = hyper::Method::GET;
            *req.uri_mut() = "http://localhost/article".parse().unwrap();
            if let Some(accept) = accept {
                req.headers_mut()
                    .insert(hyper::header::ACCEPT, HeaderValue::from_static(accept));
            }

            match req_handler.handle(&mut req, remote_addr).await {
                Ok(res) => {
                    assert_eq!(res.status(), 200);
                    let vary = res.headers().get("vary").and_then(|v| v.to_str().ok());
                    assert!(
                        vary.is_some_and(|v| v.split(',').any(|n| n.trim() == "accept")),
                        "missing `Vary: Accept` for Accept {accept:?}, got {vary:?}"
                    );
                }
                Err(err) => {
                    panic!("unexpected error: {err}")
                }
            };
        }
    }

    #[tokio::test]
    async fn markdown_enabled_no_vary_accept_on_other_assets() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        // Assets that are neither HTML nor markdown don't negotiate on `Accept`
        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/style.css".parse().unwrap();
        req.headers_mut()
            .insert(hyper::header::ACCEPT, HeaderValue::from_static("text/css"));

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                let vary = res.headers().get("vary").and_then(|v| v.to_str().ok());
                assert!(
                    !vary.is_some_and(|v| v.split(',').any(|n| n.trim() == "accept")),
                    "unexpected `Vary: Accept` on a CSS asset, got {vary:?}"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    /// Whether the response `Vary` header lists `accept` as a whole field name.
    fn varies_on_accept<B>(res: &hyper::Response<B>) -> bool {
        res.headers()
            .get("vary")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.split(',').any(|n| n.trim() == "accept"))
    }

    #[tokio::test]
    async fn markdown_enabled_vary_accept_on_html_not_modified() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        req.headers_mut()
            .insert(hyper::header::ACCEPT, HeaderValue::from_static("text/html"));
        let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.status(), 200);
        let etag = res.headers().get("etag").cloned().expect("missing ETag");
        let last_modified = res
            .headers()
            .get("last-modified")
            .cloned()
            .expect("missing Last-Modified");

        // A `304` carries no `Content-Type` but must keep the `Vary` of the `200`
        for (name, value) in [
            (hyper::header::IF_NONE_MATCH, etag),
            (hyper::header::IF_MODIFIED_SINCE, last_modified),
        ] {
            let mut req = Request::new(());
            *req.method_mut() = hyper::Method::GET;
            *req.uri_mut() = "http://localhost/article".parse().unwrap();
            req.headers_mut()
                .insert(hyper::header::ACCEPT, HeaderValue::from_static("text/html"));
            req.headers_mut().insert(name.clone(), value);

            let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
            assert_eq!(res.status(), 304, "{name}");
            assert!(
                varies_on_accept(&res),
                "missing `Vary: Accept` on a 304 for {name}, got {:?}",
                res.headers().get("vary")
            );
        }
    }

    #[cfg(feature = "mem-cache")]
    #[tokio::test]
    async fn markdown_enabled_vary_accept_on_html_not_modified_from_mem_cache() {
        let opts = fixture_settings("toml/markdown_enabled_mem_cache.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        static_web_server::mem_cache::cache::init(&mut req_handler_opts).unwrap();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let request = |if_none_match: Option<HeaderValue>| {
            let mut req = Request::new(());
            *req.method_mut() = hyper::Method::GET;
            *req.uri_mut() = "http://localhost/article.html".parse().unwrap();
            req.headers_mut()
                .insert(hyper::header::ACCEPT, HeaderValue::from_static("text/html"));
            if let Some(etag) = if_none_match {
                req.headers_mut().insert(hyper::header::IF_NONE_MATCH, etag);
            }
            req
        };

        // Drain the first response so that the cache gets populated
        let res = req_handler
            .handle(&mut request(None), remote_addr)
            .await
            .unwrap();
        let etag = res.headers().get("etag").cloned().expect("missing ETag");
        res.into_body().collect().await.unwrap();

        let res = req_handler
            .handle(&mut request(Some(etag)), remote_addr)
            .await
            .unwrap();
        assert_eq!(res.status(), 304);
        assert_eq!(
            res.headers().get("x-cache").map(|v| v.as_bytes()),
            Some(&b"HIT"[..]),
            "expected a cache hit"
        );
        assert!(
            varies_on_accept(&res),
            "missing `Vary: Accept` on a cached 304, got {:?}",
            res.headers().get("vary")
        );
    }

    #[tokio::test]
    async fn markdown_enabled_vary_accept_on_trailing_slash_redirect() {
        let opts = fixture_settings("toml/markdown_enabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        // `/guide` only has a markdown index: HTML clients get a redirect,
        // markdown clients get the variant, so both must vary on `Accept`
        for (accept, status) in [("text/html", 308), ("text/markdown", 200)] {
            let mut req = Request::new(());
            *req.method_mut() = hyper::Method::GET;
            *req.uri_mut() = "http://localhost/guide".parse().unwrap();
            req.headers_mut()
                .insert(hyper::header::ACCEPT, HeaderValue::from_static(accept));

            let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
            assert_eq!(res.status(), status, "{accept}");
            assert!(
                varies_on_accept(&res),
                "missing `Vary: Accept` for Accept {accept:?}, got {:?}",
                res.headers().get("vary")
            );
        }
    }

    #[cfg(feature = "directory-listing")]
    #[tokio::test]
    async fn markdown_enabled_vary_accept_with_custom_index_file() {
        use static_web_server::settings::cli::General;

        let opts = fixture_settings("toml/markdown_enabled.toml");
        let general = General {
            directory_listing: true,
            index_files: "index.txt".to_owned(),
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        // `/guide` has no `index.txt`: HTML clients get a redirect then a
        // listing, markdown clients get the variant, so all must vary on `Accept`
        for (uri, accept, status) in [
            ("/guide", "text/html", 308),
            ("/guide/", "text/html", 200),
            ("/guide/", "text/markdown", 200),
        ] {
            let mut req = Request::new(());
            *req.method_mut() = hyper::Method::GET;
            *req.uri_mut() = format!("http://localhost{uri}").parse().unwrap();
            req.headers_mut()
                .insert(hyper::header::ACCEPT, HeaderValue::from_static(accept));

            let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
            assert_eq!(res.status(), status, "{uri} {accept}");
            assert!(
                varies_on_accept(&res),
                "missing `Vary: Accept` for {uri} with Accept {accept:?}, got {:?}",
                res.headers().get("vary")
            );
        }
    }

    #[tokio::test]
    async fn markdown_disabled_no_vary_accept() {
        let opts = fixture_settings("toml/markdown_disabled.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/article".parse().unwrap();
        req.headers_mut().insert(
            hyper::header::ACCEPT,
            HeaderValue::from_static("text/markdown"),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                let vary = res.headers().get("vary").and_then(|v| v.to_str().ok());
                assert!(
                    !vary.is_some_and(|v| v.split(',').any(|n| n.trim() == "accept")),
                    "unexpected `Vary: Accept` when the feature is disabled, got {vary:?}"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }
}
