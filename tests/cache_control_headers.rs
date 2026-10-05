#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for the file type based `Cache-Control` headers
//! (`--cache-control-headers`).

#[cfg(test)]
pub mod tests {
    use http::HeaderMap;
    use http::header::{
        CACHE_CONTROL, IF_MODIFIED_SINCE, IF_UNMODIFIED_SINCE, LAST_MODIFIED, RANGE,
    };
    use hyper::{Body, Method, Request, Response, StatusCode};
    use std::net::SocketAddr;

    use static_web_server::settings::cli::General;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    async fn request(method: Method, uri: &str, headers: HeaderMap) -> Response<Body> {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            cache_control_headers: true,
            ..opts.general
        };
        let req_handler = fixture_req_handler(fixture_req_handler_opts(general, opts.advanced));
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(Body::empty());
        *req.method_mut() = method;
        *req.uri_mut() = ["http://localhost", uri].concat().parse().unwrap();
        *req.headers_mut() = headers;

        req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler must succeed")
    }

    fn header(name: http::HeaderName, value: &http::HeaderValue) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(name, value.clone());
        headers
    }

    #[tokio::test]
    async fn cache_control_one_year_applies_to_ok() {
        let res = request(Method::GET, "/assets/main.css", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[CACHE_CONTROL], "max-age=31536000");
    }

    #[tokio::test]
    async fn cache_control_one_year_applies_to_not_modified() {
        let res = request(Method::GET, "/assets/main.css", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::OK);
        let last_modified = res.headers()[LAST_MODIFIED].clone();
        let res = request(
            Method::GET,
            "/assets/main.css",
            header(IF_MODIFIED_SINCE, &last_modified),
        )
        .await;
        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(res.headers()[CACHE_CONTROL], "max-age=31536000");
    }

    #[tokio::test]
    async fn cache_control_one_day_applies_to_html_and_not_modified() {
        let res = request(Method::GET, "/assets/index.html", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[CACHE_CONTROL], "max-age=86400");
        let last_modified = res.headers()[LAST_MODIFIED].clone();

        let res = request(
            Method::GET,
            "/assets/index.html",
            header(IF_MODIFIED_SINCE, &last_modified),
        )
        .await;
        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(res.headers()[CACHE_CONTROL], "max-age=86400");
    }

    #[tokio::test]
    async fn cache_control_one_year_applies_to_partial_content() {
        let range = http::HeaderValue::from_static("bytes=0-3");
        let res = request(Method::GET, "/assets/main.css", header(RANGE, &range)).await;
        assert_eq!(res.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(res.headers()[CACHE_CONTROL], "max-age=31536000");
    }

    #[tokio::test]
    async fn cache_control_one_year_skips_precondition_failed() {
        let since = http::HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT");
        let res = request(
            Method::GET,
            "/assets/main.css",
            header(IF_UNMODIFIED_SINCE, &since),
        )
        .await;
        assert_eq!(res.status(), StatusCode::PRECONDITION_FAILED);
        assert_eq!(res.headers()[CACHE_CONTROL], "no-cache");
    }

    #[tokio::test]
    async fn cache_control_one_year_skips_range_not_satisfiable() {
        let range = http::HeaderValue::from_static("bytes=999999-");
        let res = request(Method::GET, "/assets/main.css", header(RANGE, &range)).await;
        assert_eq!(res.status(), StatusCode::RANGE_NOT_SATISFIABLE);
        assert_eq!(res.headers()[CACHE_CONTROL], "no-cache");
    }

    #[tokio::test]
    async fn cache_control_one_year_skips_not_found() {
        for uri in ["/missing.css", "/assets/missing.js", "/missing.png"] {
            let res = request(Method::GET, uri, HeaderMap::new()).await;
            assert_eq!(res.status(), StatusCode::NOT_FOUND, "{uri}");
            assert_eq!(res.headers()[CACHE_CONTROL], "no-cache", "{uri}");
        }
    }

    #[tokio::test]
    async fn cache_control_one_year_skips_not_found_head() {
        let res = request(Method::HEAD, "/missing.css", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(res.headers()[CACHE_CONTROL], "no-cache");
    }

    #[tokio::test]
    async fn cache_control_no_cache_applies_to_not_found_html() {
        let res = request(Method::GET, "/missing.html", HeaderMap::new()).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(res.headers()[CACHE_CONTROL], "no-cache");
    }
}
