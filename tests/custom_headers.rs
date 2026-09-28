#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for the custom HTTP headers (`[[advanced.headers]]`)
//! and their optional `status` filter.

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
