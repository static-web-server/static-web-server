#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for the custom HTTP headers (`[[advanced.headers]]`)
//! and their optional `status` filter and `scope`.

#[cfg(test)]
pub mod tests {
    use http::HeaderMap;
    use http::header::{CACHE_CONTROL, IF_MODIFIED_SINCE, LAST_MODIFIED, RANGE};
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
    #[cfg(feature = "fallback-page")]
    const FALLBACK_TOML: &str = "toml/custom_headers_fallback.toml";
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
    async fn custom_headers_apply_to_rewrite_redirects() {
        let res = get(EARLY_TOML, "/moved").await;
        assert_eq!(res.status(), StatusCode::FOUND);
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
    async fn custom_headers_default_scope_skips_rewrite_redirects() {
        let res = get(EARLY_TOML, "/moved").await;
        assert_eq!(res.status(), StatusCode::FOUND);
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

    /// A client-side route served the fallback page, the way a browser navigates to it.
    #[cfg(feature = "fallback-page")]
    async fn fallback_request(uri: &str, headers: HeaderMap) -> Response<Body> {
        request_with_opts(FALLBACK_TOML, Method::GET, uri, headers, |opts| {
            opts.page_fallback = b"<html>app</html>".to_vec();
            opts.page_fallback_uri_path = "/assets/index.html".into();
        })
        .await
    }

    #[cfg(feature = "fallback-page")]
    #[tokio::test]
    async fn custom_headers_match_the_fallback_page_by_its_path() {
        let res = fallback_request("/content/42/apply", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::OK);
        // Both differ from what the server sets on its own for this path
        assert_eq!(res.headers()[CACHE_CONTROL], "no-store");
        assert_eq!(res.headers()["x-app-shell"], "yes");
        // Matched on the page's directory, which its file name alone wouldn't give
        assert_eq!(res.headers()["x-under-assets"], "yes");
        assert!(res.headers().get("x-css-only").is_none());
    }

    #[cfg(feature = "fallback-page")]
    #[tokio::test]
    async fn custom_headers_skip_missing_assets_that_keep_their_404() {
        let mut headers = HeaderMap::new();
        headers.insert("sec-fetch-dest", "script".parse().unwrap());
        let res = fallback_request("/missing.js", headers).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert!(res.headers().get("x-app-shell").is_none());
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
