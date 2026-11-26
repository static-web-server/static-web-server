#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use http_body_util::BodyExt;
    use hyper::{
        Method, Request, Response,
        header::{HOST, LOCATION},
    };
    use std::net::SocketAddr;

    use static_web_server::body::Body;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    async fn trailing_slash_response(method: Method, uri: &str) -> Response<Body> {
        trailing_slash_response_for_host(method, uri, None).await
    }

    async fn trailing_slash_response_for_host(
        method: Method,
        uri: &str,
        host: Option<&str>,
    ) -> Response<Body> {
        let opts = fixture_settings("toml/rewrites_trailing_slash.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = method;
        *req.uri_mut() = uri.parse().unwrap();
        if let Some(host) = host {
            req.headers_mut().insert(HOST, host.parse().unwrap());
        }

        req_handler.handle(&mut req, remote_addr).await.unwrap()
    }

    const ROOT_INDEX: &str = "tests/fixtures/public/index.htm";
    const ASSETS_INDEX: &str = "tests/fixtures/public/assets/index.html";

    /// Asserts a `308` to `location`, and that `location` serves `file`.
    async fn assert_redirects_to(uri: &str, location: &str, file: &str) {
        let res = trailing_slash_response(Method::GET, uri).await;
        assert_eq!(res.status(), 308, "{uri}");
        assert_eq!(res.headers()[LOCATION], location, "{uri}");

        let target = ["http://localhost", location].concat();
        let res = trailing_slash_response(Method::GET, &target).await;
        assert_eq!(res.status(), 200, "{uri} redirects to {location}");
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            body,
            std::fs::read(file).unwrap(),
            "{location} serves {file}"
        );
    }

    #[tokio::test]
    async fn rewrites_skipped() {
        let mut opts = fixture_settings("toml/rewrites.toml");
        opts.general.index_files = "index.htm".to_owned();
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://development".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_1() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/some/error-page.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("404 Content"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_2() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/error-page/50x.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("50x Service Unavailable"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_3() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/errors/50x.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("50x Service Unavailable"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_4() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/scripts/main.js".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/javascript");

                let body = res
                    .into_body()
                    .collect()
                    .await
                    .expect("unexpected bytes error during `body` conversion")
                    .to_bytes();
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("Static Web Server"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_recursive_1() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/recursive/assets/main.css"
            .parse()
            .unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/css");
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_capture_stays_in_root() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        for uri in [
            "http://localhost/recursive/../../Cargo.toml",
            "http://localhost/recursive/%2e%2e/%2e%2e/Cargo.toml",
            "http://localhost/recursive/%2e%2e%2f%2e%2e%2fCargo.toml",
        ] {
            let mut req = Request::new(());
            *req.uri_mut() = uri.parse().unwrap();
            let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
            assert_eq!(res.status(), 404, "{uri}");
        }
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_disabled_serves_rewritten_directory() {
        let mut opts = fixture_settings("toml/rewrites_trailing_slash.toml");
        opts.general.redirect_trailing_slash = false;
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/value/manage/assets".parse().unwrap();
        let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.status(), 200);
        assert!(res.headers().get(LOCATION).is_none());
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, std::fs::read(ASSETS_INDEX).unwrap());
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_to_client_path() {
        assert_redirects_to(
            "http://localhost/value/manage",
            "/value/manage/",
            ROOT_INDEX,
        )
        .await;

        let res = trailing_slash_response(Method::HEAD, "http://localhost/value/manage").await;
        assert_eq!(res.status(), 308);
        assert_eq!(res.headers()[LOCATION], "/value/manage/");
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_keeps_client_query() {
        assert_redirects_to(
            "http://localhost/value/manage?lang=en",
            "/value/manage/?lang=en",
            ROOT_INDEX,
        )
        .await;
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_hides_rewritten_subdir() {
        assert_redirects_to(
            "http://localhost/value/manage/assets",
            "/value/manage/assets/",
            ASSETS_INDEX,
        )
        .await;
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_serves_client_path_with_slash() {
        let res = trailing_slash_response(Method::GET, "http://localhost/value/manage/").await;
        assert_eq!(res.status(), 200);
        assert!(res.headers().get(LOCATION).is_none());
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_skips_rewritten_file() {
        let res = trailing_slash_response(Method::GET, "http://localhost/assets").await;
        assert_eq!(
            res.status(),
            200,
            "a route rewritten to a file must not redirect, even if it names a directory"
        );
        assert!(res.headers().get(LOCATION).is_none());
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_serves_exact_rewrite_with_slash() {
        let res = trailing_slash_response(Method::GET, "http://localhost/exact").await;
        assert_eq!(
            res.status(),
            200,
            "`/exact/` matches no rule, so `/exact` must not redirect there"
        );
        assert!(res.headers().get(LOCATION).is_none());
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_exact_rewrite_to_destination() {
        assert_redirects_to("http://localhost/exact-no-slash", "/assets/", ASSETS_INDEX).await;
        assert_redirects_to("http://localhost/slashed/", "/assets/", ASSETS_INDEX).await;
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_once() {
        assert_redirects_to("http://localhost/app/x", "/assets/", ASSETS_INDEX).await;
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_client_slash_to_destination() {
        assert_redirects_to("http://localhost/listing/", "/assets/", ASSETS_INDEX).await;
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_host_rule_to_client_path() {
        for (uri, host) in [
            ("/tenant", Some("Tenant.Example:8080")),
            ("http://tenant.example/tenant", None),
            ("http://tenant.example/tenant", Some("localhost")),
        ] {
            let res = trailing_slash_response_for_host(Method::GET, uri, host).await;
            assert_eq!(res.status(), 308, "{uri} with host {host:?}");
            assert_eq!(
                res.headers()[LOCATION],
                "/tenant/",
                "{uri} with host {host:?}"
            );
        }

        let res =
            trailing_slash_response_for_host(Method::GET, "/tenant/", Some("tenant.example")).await;
        assert_eq!(res.status(), 200);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, std::fs::read(ASSETS_INDEX).unwrap());

        let res = trailing_slash_response_for_host(Method::GET, "/tenant", Some("localhost")).await;
        assert_eq!(res.status(), 404, "the rule belongs to another host");
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_to_client_path_across_virtual_hosts() {
        // The rewrite replaces `Host`, and its rule only applies to the one the client sent
        let host = Some("front.example");
        let res = trailing_slash_response_for_host(Method::GET, "/vhost", host).await;
        assert_eq!(res.status(), 308);
        assert_eq!(res.headers()[LOCATION], "/vhost/");

        let res = trailing_slash_response_for_host(Method::GET, "/vhost/", host).await;
        assert_eq!(res.status(), 200);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, std::fs::read(ASSETS_INDEX).unwrap());
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_skips_client_path_with_redirect_rule() {
        let res = trailing_slash_response(Method::GET, "http://localhost/moved/").await;
        assert_eq!(res.status(), 301);
        assert_eq!(res.headers()[LOCATION], "/elsewhere");

        // `/moved/` is redirected elsewhere, so it can't serve the directory
        assert_redirects_to("http://localhost/moved", "/assets/", ASSETS_INDEX).await;
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_padded_client_path() {
        // `/./` segments resolve to `/assets` and pad the path to the 8 KiB cap, which is
        // on the path without them
        let prefix = "/value/manage";
        let padding = "/.".repeat((8 * 1024 - prefix.len() - "/assets".len()) / 2);
        let path = [prefix, &padding, "/assets"].concat();
        assert_eq!(path.len(), 8 * 1024);

        let res = trailing_slash_response(Method::GET, &["http://localhost", &path].concat()).await;
        assert_eq!(res.status(), 308);
        assert_eq!(res.headers()[LOCATION], [&path, "/"].concat());
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_spelled_with_dot_segment_is_a_directory() {
        // `/tenant/.` names the directory, as `/tenant/` does, so no redirect to add a slash
        let host = Some("tenant.example");
        for uri in ["/tenant/.", "/tenant/%2E", "/tenant%2F"] {
            let res = trailing_slash_response_for_host(Method::GET, uri, host).await;
            assert_eq!(res.status(), 200, "{uri}");
            let body = res.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, std::fs::read(ASSETS_INDEX).unwrap(), "{uri}");
        }
    }

    #[tokio::test]
    async fn rewrites_trailing_slash_location_stays_on_host() {
        for (uri, location) in [
            ("http://localhost//evil.example/docs", "/evil.example/docs/"),
            (
                "http://localhost/\\evil.example/docs",
                "/%5Cevil.example/docs/",
            ),
            (
                "http://localhost/\\/evil.example/docs",
                "/%5C/evil.example/docs/",
            ),
        ] {
            assert_redirects_to(uri, location, ASSETS_INDEX).await;
        }
    }

    #[cfg(feature = "mem-cache")]
    #[tokio::test]
    async fn rewrites_trailing_slash_redirects_on_cached_index() {
        let opts = fixture_settings("toml/rewrites_trailing_slash_mem_cache.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        static_web_server::mem_cache::cache::init(&mut req_handler_opts).unwrap();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        for _ in 0..2 {
            let mut req = Request::new(());
            *req.uri_mut() = "http://localhost/value/manage/".parse().unwrap();
            let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
            assert_eq!(res.status(), 200);
            res.into_body().collect().await.unwrap();
        }

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/value/manage/".parse().unwrap();
        let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.headers()["x-cache"], "HIT", "the index must be cached");

        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/value/manage".parse().unwrap();
        let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.status(), 308);
        assert_eq!(res.headers()[LOCATION], "/value/manage/");
    }

    #[cfg(feature = "mem-cache")]
    #[tokio::test]
    async fn rewrites_host_rule_cached_file_stays_on_its_host() {
        let opts = fixture_settings("toml/rewrites_trailing_slash_mem_cache.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        static_web_server::mem_cache::cache::init(&mut req_handler_opts).unwrap();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let get = |host: &'static str| {
            let mut req = Request::new(());
            *req.uri_mut() = "/app.js".parse().unwrap();
            req.headers_mut().insert(HOST, host.parse().unwrap());
            req
        };

        for _ in 0..2 {
            let res = req_handler
                .handle(&mut get("tenant.example"), remote_addr)
                .await
                .unwrap();
            assert_eq!(res.status(), 200);
            res.into_body().collect().await.unwrap();
        }
        let res = req_handler
            .handle(&mut get("Tenant.Example:8080"), remote_addr)
            .await
            .unwrap();
        assert_eq!(res.headers()["x-cache"], "HIT", "the file must be cached");

        let res = req_handler
            .handle(&mut get("localhost"), remote_addr)
            .await
            .unwrap();
        assert_eq!(res.status(), 404, "the rule belongs to another host");
    }
}
