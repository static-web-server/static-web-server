#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for the custom HTTP headers (`[[advanced.headers]]`)
//! and their optional `status` filter and `scope`.

#[cfg(test)]
pub mod tests {
    use http::HeaderMap;
    use http::header::{CACHE_CONTROL, IF_MODIFIED_SINCE, LAST_MODIFIED, RANGE, VARY};
    use hyper::{Method, Request, Response, StatusCode};
    use std::net::SocketAddr;
    use std::path::PathBuf;

    use static_web_server::Settings;
    use static_web_server::body::Body;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    const STATUS_TOML: &str = "toml/custom_headers_status.toml";
    const ORDER_TOML: &str = "toml/custom_headers_status_order.toml";
    const EARLY_TOML: &str = "toml/custom_headers_early.toml";
    const MAINTENANCE_TOML: &str = "toml/custom_headers_maintenance.toml";
    const SPELLING_TOML: &str = "toml/custom_headers_path_spelling.toml";
    const PATHS_TOML: &str = "toml/custom_headers_paths.toml";
    const SERVED_TOML: &str = "toml/custom_headers_served_file.toml";
    const QUOTED_TOML: &str = "toml/custom_headers_quoted_source.toml";
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";

    async fn request(
        fixture_toml: &str,
        method: Method,
        uri: &str,
        headers: HeaderMap,
    ) -> Response<Body> {
        let opts = fixture_settings(fixture_toml);
        let req_handler =
            fixture_req_handler(fixture_req_handler_opts(opts.general, opts.advanced));
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = method;
        *req.uri_mut() = ["http://localhost", uri].concat().parse().unwrap();
        *req.headers_mut() = headers;

        req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler must succeed")
    }

    async fn request_with_opts(
        fixture_toml: &str,
        method: Method,
        uri: &str,
        headers: HeaderMap,
        configure: impl FnOnce(&mut static_web_server::handler::RequestHandlerOpts),
    ) -> Response<Body> {
        let opts = fixture_settings(fixture_toml);
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        configure(&mut req_handler_opts);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = method;
        *req.uri_mut() = ["http://localhost", uri].concat().parse().unwrap();
        *req.headers_mut() = headers;

        req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler must succeed")
    }

    /// A CORS preflight from an origin that is not allowed: rejected before the file handler.
    async fn cors_rejected_preflight(fixture_toml: &str) -> Response<Body> {
        let mut headers = HeaderMap::new();
        headers.insert("origin", "http://localhost".parse().unwrap());
        headers.insert("access-control-request-method", "GET".parse().unwrap());
        request_with_opts(
            fixture_toml,
            Method::OPTIONS,
            "/assets/main.css",
            headers,
            |opts| opts.cors = static_web_server::cors::new("https://allowed.example", "", ""),
        )
        .await
    }

    async fn get(fixture_toml: &str, uri: &str) -> Response<Body> {
        request(fixture_toml, Method::GET, uri, HeaderMap::new()).await
    }

    fn header(name: http::HeaderName, value: &http::HeaderValue) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(name, value.clone());
        headers
    }

    fn settings_error(fixture_toml: &str) -> String {
        let f = PathBuf::from("tests/fixtures").join(fixture_toml);
        match Settings::get_unparsed(
            false,
            &["static-web-server", "--config-file", f.to_str().unwrap()],
        ) {
            Ok(_) => panic!("settings for {fixture_toml} must be rejected"),
            Err(err) => format!("{err:#}"),
        }
    }

    #[tokio::test]
    async fn custom_headers_match_every_spelling_of_a_path() {
        for uri in [
            "/assets/main.js",
            "/assets/./main.js",
            "//assets/main.js",
            "/assets//main.js",
            "/assets/%6Dain.js",
            "/assets/%6dain.js",
            "/assets%2Fmain.js",
            "/assets/%2e/main.js",
            "/./assets/%2E%2F/main.js",
        ] {
            let res = get(SPELLING_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-exact"], "1", "{uri}");
            assert_eq!(res.headers()["x-glob"], "1", "{uri}");
        }

        let res = get(SPELLING_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), 200);
        assert!(res.headers().get("x-exact").is_none());
        assert_eq!(res.headers()["x-glob"], "1");
    }

    #[tokio::test]
    async fn custom_headers_source_matches_in_any_spelling() {
        for uri in [
            "/sp%C3%A9cial-direct%C3%B6ry.net/sp%C3%A9cial%20file.txt~",
            "/sp%c3%a9cial-direct%c3%b6ry.net/sp%c3%a9cial%20file.txt~",
            "/spécial-directöry.net/spécial%20file.txt~",
        ] {
            let res = get(SPELLING_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-characters"], "1", "{uri}");
            assert_eq!(res.headers()["x-escapes"], "1", "{uri}");
        }
    }

    #[cfg(any(target_os = "macos", windows))]
    #[tokio::test]
    async fn custom_headers_match_any_letter_case_where_files_do() {
        for uri in ["/ASSETS/main.js", "/Assets/Main.JS"] {
            let res = get(SPELLING_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-exact"], "1", "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_index_file_of_a_directory_matches_by_name() {
        // A directory request is matched as its index file, in the same spelling
        for uri in ["/assets/", "//assets/", "/%61ssets/"] {
            let res = get(SPELLING_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-glob"], "1", "{uri}");
        }
    }

    /// Requests `uri` with `redirect-trailing-slash` off.
    async fn get_without_slash_redirect(fixture_toml: &str, uri: &str) -> Response<Body> {
        request_with_opts(fixture_toml, Method::GET, uri, HeaderMap::new(), |opts| {
            opts.redirect_trailing_slash = false;
        })
        .await
    }

    #[tokio::test]
    async fn custom_headers_directory_request_matches_its_index_file() {
        for uri in ["/assets/", "/assets/index.html", "//assets/./"] {
            let res = get(PATHS_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-index"], "1", "{uri}");
            assert!(res.headers().get("x-file").is_none(), "{uri}");
        }
        for uri in ["/", "/index.htm"] {
            let res = get(PATHS_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-root-index"], "1", "{uri}");
            assert!(res.headers().get("x-index").is_none(), "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_trailing_slash_redirect_is_not_matched_as_its_index() {
        let res = get(PATHS_TOML, "/assets").await;
        assert_eq!(res.status(), 308);
        assert!(res.headers().get("x-index").is_none());
    }

    #[tokio::test]
    async fn custom_headers_without_slash_redirect_match_files_by_their_path() {
        for (uri, header) in [
            ("/assets/main.js", "x-file"),
            ("/index.htm", "x-root-index"),
            ("/", "x-root-index"),
            ("/assets/index.html", "x-index"),
        ] {
            let res = get_without_slash_redirect(PATHS_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()[header], "1", "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_without_slash_redirect_match_directory_by_its_index_file() {
        for uri in ["/assets", "/assets/"] {
            let res = get_without_slash_redirect(PATHS_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-index"], "1", "{uri}");
            assert!(res.headers().get("x-file").is_none(), "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_source_glob_forms() {
        for (uri, expected) in [
            ("/assets/main.css", ["x-star", "x-alternation"].as_slice()),
            (
                "/assets/main.js",
                &["x-alternation", "x-class", "x-question"],
            ),
            ("/assets/tiny.js", &["x-alternation", "x-class"]),
            ("/assets/index.html", &[]),
            ("/index.htm", &[]),
        ] {
            let res = get(PATHS_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            for header in ["x-star", "x-alternation", "x-class", "x-question"] {
                assert_eq!(
                    res.headers().contains_key(header),
                    expected.contains(&header),
                    "{header} on {uri}"
                );
            }
        }
    }

    #[tokio::test]
    async fn custom_headers_entry_sets_several_headers_and_later_entry_wins() {
        let res = get(PATHS_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers()["x-first"],
            "c",
            "names are compared in any case"
        );
        assert_eq!(res.headers().get_all("x-first").iter().count(), 1);
        assert_eq!(res.headers()["x-second"], "b");
        assert_eq!(
            res.headers()[CACHE_CONTROL],
            "no-store",
            "a custom header replaces the one the server sets"
        );
        assert_eq!(res.headers().get_all(CACHE_CONTROL).iter().count(), 1);
        assert_eq!(res.headers()[VARY], "x-custom");
    }

    #[tokio::test]
    async fn custom_headers_apply_to_options() {
        let res = request(
            PATHS_TOML,
            Method::OPTIONS,
            "/assets/main.js",
            HeaderMap::new(),
        )
        .await;
        assert_eq!(res.status(), 204);
        assert_eq!(res.headers()["x-file"], "1");
    }

    #[tokio::test]
    async fn custom_headers_match_the_path_a_request_is_rewritten_to() {
        for uri in ["/app/main.js", "/app/./%6Dain.js"] {
            let res = get(PATHS_TOML, uri).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-file"], "1", "{uri}");
            assert!(res.headers().get("x-client-path").is_none(), "{uri}");
        }

        let res = get(PATHS_TOML, "/app/").await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["x-index"], "1");
    }

    #[tokio::test]
    async fn custom_headers_apply_to_virtual_hosts() {
        let get_from = |uri: &'static str| async move {
            let opts = fixture_settings(PATHS_TOML);
            let req_handler =
                fixture_req_handler(fixture_req_handler_opts(opts.general, opts.advanced));
            let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
            let mut req = Request::new(());
            *req.uri_mut() = uri.parse().unwrap();
            req_handler.handle(&mut req, remote_addr).await.unwrap()
        };

        let res = get_from("http://docs.example/test.html").await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["x-vhost-file"], "1");

        let res = get_from("http://DOCS.example/style.css").await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["x-star"], "1");

        let res = get_from("http://localhost/test.html").await;
        assert_eq!(res.status(), 404, "the file is in the virtual host only");
        assert_eq!(res.headers()["x-vhost-file"], "1");
    }

    /// The `x-*` response headers that the fixture entries set, sorted.
    fn custom_names(res: &Response<Body>) -> Vec<&str> {
        const SERVER_SET: [&str; 3] = ["x-content-type-options", "x-frame-options", "x-cache"];
        let mut names: Vec<&str> = res
            .headers()
            .keys()
            .map(|name| name.as_str())
            .filter(|name| name.starts_with("x-") && !SERVER_SET.contains(name))
            .collect();
        names.sort_unstable();
        names
    }

    /// A root with a directory that has a `.html` sibling and no index file, a directory
    /// to list, and a file with sub-delimiters in its name.
    fn served_file_root() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("docs")).unwrap();
        std::fs::write(dir.path().join("docs.html"), "sibling").unwrap();
        std::fs::create_dir(dir.path().join("list")).unwrap();
        std::fs::write(dir.path().join("list/a.txt"), "a").unwrap();
        std::fs::write(dir.path().join("a+b,c.txt"), "sub-delims").unwrap();
        dir
    }

    /// Requests `uri` from `SERVED_TOML` with `root` as the root directory.
    async fn get_from_root(root: &std::path::Path, uri: &str, listing: bool) -> Response<Body> {
        let root = root.canonicalize().unwrap();
        request_with_opts(SERVED_TOML, Method::GET, uri, HeaderMap::new(), |opts| {
            opts.root_dir = root;
            opts.index_files = vec!["index.html".to_owned()];
            #[cfg(feature = "directory-listing")]
            {
                opts.dir_listing = listing;
            }
            #[cfg(not(feature = "directory-listing"))]
            let _ = listing;
        })
        .await
    }

    #[tokio::test]
    async fn custom_headers_reach_a_file_served_through_the_html_fallback() {
        for method in [Method::GET, Method::HEAD] {
            for redirect_trailing_slash in [true, false] {
                let res = request_with_opts(
                    SERVED_TOML,
                    method.clone(),
                    "/404",
                    HeaderMap::new(),
                    |opts| opts.redirect_trailing_slash = redirect_trailing_slash,
                )
                .await;
                assert_eq!(res.status(), 200);
                assert_eq!(custom_names(&res), ["x-file", "x-html", "x-request"]);
            }
        }

        let res = get(SERVED_TOML, "/404.html").await;
        assert_eq!(custom_names(&res), ["x-file", "x-html"]);
    }

    #[tokio::test]
    async fn custom_headers_directory_source_matches_the_directory_request() {
        for method in [Method::GET, Method::HEAD] {
            let res = request(SERVED_TOML, method, "/assets/", HeaderMap::new()).await;
            assert_eq!(res.status(), 200);
            assert_eq!(custom_names(&res), ["x-directory", "x-html"]);
        }

        let res = get(SERVED_TOML, "/assets/index.html").await;
        assert_eq!(custom_names(&res), ["x-html"]);

        let res = get(SERVED_TOML, "/").await;
        assert_eq!(res.status(), 200);
        assert_eq!(custom_names(&res), ["x-root"]);
    }

    #[tokio::test]
    async fn custom_headers_source_class_of_one_character_matches_it() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["a b.txt", "a2b.txt", "[x].txt", "5x].txt"] {
            std::fs::write(dir.path().join(name), name).unwrap();
        }
        let root = dir.path().canonicalize().unwrap();

        for (uri, names) in [
            ("/a%20b.txt", vec!["x-space"]),
            ("/a2b.txt", vec![]),
            ("/%5Bx%5D.txt", vec!["x-brackets"]),
            ("/5x%5D.txt", vec![]),
        ] {
            let res = request_with_opts(QUOTED_TOML, Method::GET, uri, HeaderMap::new(), |opts| {
                opts.root_dir = root.clone();
            })
            .await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(custom_names(&res), names, "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_reach_the_html_sibling_served_for_a_directory() {
        let root = served_file_root();
        let res = get_from_root(root.path(), "/docs/", false).await;
        assert_eq!(res.status(), 200);
        assert_eq!(custom_names(&res), ["x-html", "x-sibling"]);
    }

    #[cfg(feature = "directory-listing")]
    #[tokio::test]
    async fn custom_headers_apply_to_a_directory_listing() {
        let root = served_file_root();
        let res = get_from_root(root.path(), "/list/", true).await;
        assert_eq!(res.status(), 200);
        assert_eq!(
            custom_names(&res),
            ["x-html", "x-listing"],
            "a listing is matched as its directory and as the index file it stands in for"
        );
    }

    #[tokio::test]
    async fn custom_headers_reach_a_negotiated_markdown_variant() {
        let get_article = |accept: Option<&'static str>| async move {
            let opts = fixture_settings(SERVED_TOML);
            let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
            req_handler_opts.accept_markdown = true;
            let req_handler = fixture_req_handler(req_handler_opts);
            let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
            let mut req = Request::new(());
            *req.uri_mut() = "http://docs.example/article.html".parse().unwrap();
            if let Some(accept) = accept {
                req.headers_mut()
                    .insert(http::header::ACCEPT, accept.parse().unwrap());
            }
            req_handler.handle(&mut req, remote_addr).await.unwrap()
        };

        let res = get_article(Some("text/markdown")).await;
        assert_eq!(res.status(), 200);
        assert_eq!(custom_names(&res), ["x-html", "x-markdown"]);

        let res = get_article(None).await;
        assert_eq!(res.status(), 200);
        assert_eq!(custom_names(&res), ["x-html"]);
    }

    #[tokio::test]
    async fn custom_headers_trailing_slash_redirect_serves_no_file() {
        for uri in ["/assets", "/assets/."] {
            let res = get(SERVED_TOML, uri).await;
            assert_eq!(res.status(), 308, "{uri}");
            assert!(res.headers().get("x-html").is_none(), "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_file_requested_as_a_directory_gets_none_of_its_headers() {
        let res = get(SERVED_TOML, "/404.html/").await;
        assert_eq!(res.status(), 404);
        assert!(custom_names(&res).is_empty());
    }

    #[tokio::test]
    async fn custom_headers_keep_every_value_of_a_header() {
        let res = get(SERVED_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), 200);
        let values = |name: &str| -> Vec<String> {
            let values = res.headers().get_all(name).iter();
            values.map(|v| v.to_str().unwrap().to_owned()).collect()
        };
        assert_eq!(
            values("link"),
            ["</a.css>; rel=preload", "</b.css>; rel=preload"]
        );
        assert_eq!(values("x-values"), ["new-1", "new-2"]);
    }

    #[tokio::test]
    async fn custom_headers_source_escapes_stand_for_their_characters() {
        let res = get(SERVED_TOML, "/assets/main.js").await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["x-escaped"], "1");

        let root = served_file_root();
        for uri in ["/a+b,c.txt", "/a%2Bb%2Cc.txt", "/a%2bb,c.txt"] {
            let res = get_from_root(root.path(), uri, false).await;
            assert_eq!(res.status(), 200, "{uri}");
            assert_eq!(res.headers()["x-sub-delims"], "1", "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_early_response_matches_every_spelling_of_a_path() {
        for uri in ["/old", "//old", "/%6Fld", "/./old"] {
            let res = get(SERVED_TOML, uri).await;
            assert_eq!(res.status(), 301, "{uri}");
            assert_eq!(res.headers()["x-early"], "1", "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_long_path_is_matched() {
        // Header sources have no length cap of their own: without redirects or rewrites,
        // a path as long as a URI can be is matched
        let uri = ["/assets/", &"a".repeat(60 * 1024)].concat();
        let res = get(STATUS_TOML, &uri).await;
        assert_eq!(res.status(), 404);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn custom_headers_apply_to_compressed_responses() {
        use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING};

        // Compressed on the fly; files under 200 bytes are not
        let res = request_with_opts(
            PATHS_TOML,
            Method::GET,
            "/assets/",
            header(ACCEPT_ENCODING, &"gzip".parse().unwrap()),
            |opts| opts.compression = true,
        )
        .await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()[CONTENT_ENCODING], "gzip");
        assert_eq!(res.headers()["x-index"], "1");

        // A pre-compressed twin is matched by the path of the file it stands for
        let res = request_with_opts(
            PATHS_TOML,
            Method::GET,
            "/assets/tiny.js",
            header(ACCEPT_ENCODING, &"br".parse().unwrap()),
            |opts| opts.compression_static = true,
        )
        .await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()[CONTENT_ENCODING], "br");
        assert_eq!(res.headers()["x-class"], "1");
    }

    #[cfg(feature = "mem-cache")]
    #[tokio::test]
    async fn custom_headers_apply_to_cached_files() {
        let opts = fixture_settings("toml/custom_headers_mem_cache.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        static_web_server::mem_cache::cache::init(&mut req_handler_opts).unwrap();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        for (uri, header) in [("/assets/main.js", "x-file"), ("/assets/", "x-index")] {
            let mut cache_status = None;
            for _ in 0..3 {
                let mut req = Request::new(());
                *req.uri_mut() = uri.parse().unwrap();
                let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
                assert_eq!(res.status(), 200, "{uri}");
                assert_eq!(res.headers()[header], "1", "{uri}");
                cache_status = res.headers().get("x-cache").cloned();
                http_body_util::BodyExt::collect(res.into_body())
                    .await
                    .unwrap();
            }
            assert_eq!(cache_status.unwrap(), "HIT", "{uri} must be cached");
        }
    }

    #[tokio::test]
    async fn custom_headers_status_filter_applies_to_matching_status() {
        let res = get(STATUS_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-status-filtered"], "1");
        assert_eq!(res.headers()[CACHE_CONTROL], IMMUTABLE);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[tokio::test]
    async fn custom_headers_status_filter_applies_to_head() {
        let res = request(
            STATUS_TOML,
            Method::HEAD,
            "/assets/main.css",
            HeaderMap::new(),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-status-filtered"], "1");
    }

    #[tokio::test]
    async fn custom_headers_status_filter_applies_to_partial_content() {
        let range = http::HeaderValue::from_static("bytes=0-3");
        let res = request(
            STATUS_TOML,
            Method::GET,
            "/assets/main.css",
            header(RANGE, &range),
        )
        .await;
        assert_eq!(res.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(res.headers()["x-status-filtered"], "1");
    }

    #[tokio::test]
    async fn custom_headers_status_filter_applies_to_not_modified() {
        let res = get(STATUS_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), StatusCode::OK);
        let last_modified = res.headers()[LAST_MODIFIED].clone();

        let res = request(
            STATUS_TOML,
            Method::GET,
            "/assets/main.css",
            header(IF_MODIFIED_SINCE, &last_modified),
        )
        .await;
        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(res.headers()["x-status-filtered"], "1");
    }

    #[tokio::test]
    async fn custom_headers_status_filter_skips_other_status() {
        let res = get(STATUS_TOML, "/assets/missing.css").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert!(res.headers().get("x-status-filtered").is_none());
    }

    #[tokio::test]
    async fn custom_headers_status_filter_never_matching_status() {
        for uri in ["/assets/main.css", "/assets/missing.css"] {
            let res = get(STATUS_TOML, uri).await;
            assert!(res.headers().get("x-teapot").is_none(), "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_without_status_apply_to_every_status() {
        let res = get(STATUS_TOML, "/assets/missing.css").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[tokio::test]
    async fn custom_headers_status_filter_overrides_earlier_rule() {
        let res = get(ORDER_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[CACHE_CONTROL], IMMUTABLE);

        let res = get(ORDER_TOML, "/assets/missing.css").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(res.headers()[CACHE_CONTROL], "no-store");
    }

    #[tokio::test]
    async fn custom_headers_apply_to_redirects() {
        let res = get(EARLY_TOML, "/old").await;
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.headers()[CACHE_CONTROL], "public, max-age=86400");
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[tokio::test]
    async fn custom_headers_apply_to_method_not_allowed() {
        let res = request(EARLY_TOML, Method::POST, "/old", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
        assert!(res.headers().get(CACHE_CONTROL).is_none());
    }

    #[tokio::test]
    async fn custom_headers_apply_to_health() {
        let res = get(EARLY_TOML, "/livez").await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[tokio::test]
    async fn custom_headers_apply_to_maintenance() {
        let res = get(MAINTENANCE_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(res.headers()["retry-after"], "120");
        assert!(res.headers().get("x-default-scope").is_none());
    }

    #[tokio::test]
    async fn custom_headers_default_scope_skips_redirects() {
        let res = get(EARLY_TOML, "/old").await;
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert!(res.headers().get("x-default-scope").is_none());
        assert!(res.headers().get("x-files-scope").is_none());
    }

    #[tokio::test]
    async fn custom_headers_default_scope_skips_health_and_method_check() {
        let res = get(EARLY_TOML, "/readyz").await;
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().get("x-default-scope").is_none());

        let res = request(EARLY_TOML, Method::POST, "/old", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert!(res.headers().get("x-files-scope").is_none());
    }

    #[tokio::test]
    async fn custom_headers_files_scope_applies_to_files_and_not_found() {
        for (uri, status) in [
            ("/assets/main.css", StatusCode::OK),
            ("/assets/missing.css", StatusCode::NOT_FOUND),
        ] {
            let res = get(EARLY_TOML, uri).await;
            assert_eq!(res.status(), status, "{uri}");
            assert_eq!(res.headers()["x-default-scope"], "1", "{uri}");
            assert_eq!(res.headers()["x-files-scope"], "1", "{uri}");
            assert_eq!(res.headers()["x-custom-header"], "every-response", "{uri}");
        }
    }

    #[tokio::test]
    async fn custom_headers_all_scope_applies_to_files() {
        let res = get(EARLY_TOML, "/assets/main.css").await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[tokio::test]
    async fn custom_headers_all_scope_status_filter_skips_other_status() {
        let res = get(EARLY_TOML, "/old").await;
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.headers()[CACHE_CONTROL], "public, max-age=86400");

        let res = get(EARLY_TOML, "/moved").await;
        assert_eq!(res.status(), StatusCode::FOUND);
        assert!(res.headers().get(CACHE_CONTROL).is_none());
    }

    #[tokio::test]
    async fn custom_headers_all_scope_applies_to_cors_rejection() {
        let res = cors_rejected_preflight(EARLY_TOML).await;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[tokio::test]
    async fn custom_headers_default_scope_skips_cors_rejection() {
        let res = cors_rejected_preflight(EARLY_TOML).await;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        assert!(res.headers().get("x-default-scope").is_none());
        assert!(res.headers().get("x-files-scope").is_none());
    }

    #[cfg(feature = "metrics")]
    #[tokio::test]
    async fn custom_headers_all_scope_applies_to_metrics() {
        let res = request_with_opts(
            EARLY_TOML,
            Method::GET,
            "/metrics",
            HeaderMap::new(),
            |opts| opts.metrics_enabled = true,
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-custom-header"], "every-response");
    }

    #[cfg(feature = "metrics")]
    #[tokio::test]
    async fn custom_headers_default_scope_skips_metrics() {
        let res = request_with_opts(
            EARLY_TOML,
            Method::GET,
            "/metrics",
            HeaderMap::new(),
            |opts| opts.metrics_enabled = true,
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().get("x-default-scope").is_none());
        assert!(res.headers().get("x-files-scope").is_none());
    }

    #[cfg(feature = "basic-auth")]
    #[tokio::test]
    async fn custom_headers_apply_to_basic_auth_challenge() {
        let opts = fixture_settings(EARLY_TOML);
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        req_handler_opts.basic_auth =
            "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q".into();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/assets/main.css".parse().unwrap();

        let res = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler must succeed");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        assert!(res.headers().get("www-authenticate").is_some());
        assert_eq!(res.headers()["x-custom-header"], "every-response");
        assert!(res.headers().get("x-default-scope").is_none());
    }

    #[test]
    fn custom_headers_invalid_source_is_rejected() {
        let err = settings_error("toml/custom_headers_source_invalid.toml");
        assert!(
            err.contains("can not compile glob pattern for header source: /assets/{unclosed"),
            "{err}"
        );
    }

    #[test]
    fn custom_headers_source_class_of_encoded_characters_is_rejected() {
        let err = settings_error("toml/custom_headers_source_class_invalid.toml");
        assert!(
            err.contains("can not compile glob pattern for header source: /[a b]x"),
            "{err}"
        );
        assert!(
            err.contains("class [a b] matches a single character")
                && err.contains("spells ' ' as %20"),
            "{err}"
        );
    }

    #[test]
    fn custom_headers_invalid_name_or_value_is_rejected() {
        for fixture in [
            "toml/custom_headers_name_invalid.toml",
            "toml/custom_headers_value_invalid.toml",
        ] {
            let err = settings_error(fixture);
            assert!(err.contains("headers"), "{fixture}: {err}");
        }
    }

    #[test]
    fn custom_headers_missing_headers_table_is_rejected() {
        let err = settings_error("toml/custom_headers_missing_headers.toml");
        assert!(err.contains("headers"), "{err}");
    }

    #[test]
    fn custom_headers_unknown_scope_is_rejected() {
        let err = settings_error("toml/custom_headers_scope_invalid.toml");
        assert!(err.contains("everything"), "{err}");
    }

    #[test]
    fn custom_headers_status_empty_list_is_rejected() {
        let err = settings_error("toml/custom_headers_status_empty.toml");
        assert!(err.contains("empty status list"), "{err}");
    }

    #[test]
    fn custom_headers_status_invalid_code_is_rejected() {
        let err = settings_error("toml/custom_headers_status_invalid.toml");
        assert!(err.contains("invalid status code 20"), "{err}");
    }
}
