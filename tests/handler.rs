#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use headers::HeaderValue;
    use hyper::{Method, Request};
    use std::net::SocketAddr;

    use static_web_server::exts::http::MethodExt;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    #[tokio::test]
    async fn custom_headers_apply_for_dir() {
        let opts = fixture_settings("toml/handler.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-deflate",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd"
                ))]
                #[cfg(feature = "compression")]
                assert_eq!(
                    res.headers().get("vary"),
                    Some(&HeaderValue::from_static("accept-encoding"))
                );

                assert_eq!(
                    res.headers().get("server"),
                    Some(&HeaderValue::from_static("Static Web Server"))
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn custom_headers_apply_for_file() {
        let opts = fixture_settings("toml/handler.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/index.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-deflate",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd"
                ))]
                #[cfg(feature = "compression")]
                assert_eq!(
                    res.headers().get("vary"),
                    Some(&HeaderValue::from_static("accept-encoding"))
                );

                assert_eq!(
                    res.headers().get("cache-control"),
                    Some(&HeaderValue::from_static("no-cache"))
                );
                assert_eq!(
                    res.headers().get("server"),
                    Some(&HeaderValue::from_static("Static Web Server"))
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn check_allowed_methods() {
        let opts = fixture_settings("toml/handler.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let methods = [
            Method::CONNECT,
            Method::DELETE,
            Method::GET,
            Method::HEAD,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ];
        for method in methods {
            let mut req = Request::new(());
            *req.method_mut() = method.clone();
            *req.uri_mut() = "http://localhost/index.html".parse().unwrap();

            match req_handler.handle(&mut req, remote_addr).await {
                Ok(res) => {
                    if method.is_allowed() {
                        assert_eq!(res.status(), 200);
                        assert_eq!(
                            res.headers().get("content-type"),
                            Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                        );

                        #[cfg(feature = "compression")]
                        assert_eq!(
                            res.headers().get("vary"),
                            Some(&HeaderValue::from_static("accept-encoding"))
                        );

                        assert_eq!(
                            res.headers().get("server"),
                            Some(&HeaderValue::from_static("Static Web Server"))
                        );
                    } else {
                        assert_eq!(res.status(), 405);
                    }
                }
                Err(err) => {
                    panic!("unexpected error: {err}")
                }
            };
        }
    }

    #[cfg(all(feature = "basic-auth", feature = "metrics"))]
    #[tokio::test]
    async fn metrics_requires_basic_auth_when_enabled() {
        let opts = fixture_settings("toml/handler.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        req_handler_opts.basic_auth =
            "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q".into();
        req_handler_opts.metrics_enabled = true;
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = "http://localhost/metrics".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 401);
                assert!(res.headers().get("www-authenticate").is_some());
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[cfg(all(feature = "basic-auth", feature = "metrics"))]
    #[tokio::test]
    async fn metrics_allows_valid_basic_auth_when_enabled() {
        let opts = fixture_settings("toml/handler.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        req_handler_opts.basic_auth =
            "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q".into();
        req_handler_opts.metrics_enabled = true;
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = "http://localhost/metrics".parse().unwrap();
        req.headers_mut()
            .insert("authorization", HeaderValue::from_static("Basic anE6anE="));

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/plain; charset=utf-8"))
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[cfg(feature = "fallback-page")]
    #[tokio::test]
    async fn page_fallback_does_not_log_not_found_warning() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        use tracing::{Event, Level, Subscriber};
        use tracing_subscriber::{Layer, layer::Context, prelude::*};

        #[derive(Clone, Default)]
        struct WarningCounter(Arc<AtomicUsize>);

        impl<S> Layer<S> for WarningCounter
        where
            S: Subscriber,
        {
            fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
                if *event.metadata().level() == Level::WARN {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        let opts = fixture_settings("toml/handler.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        req_handler_opts.page_fallback = b"fallback".to_vec();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let warnings = WarningCounter::default();
        let subscriber = tracing_subscriber::registry().with(warnings.clone());
        let _guard = tracing::subscriber::set_default(subscriber);

        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = "http://localhost/missing-route".parse().unwrap();

        let resp = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("fallback request should succeed");

        assert_eq!(resp.status(), 200);
        assert_eq!(warnings.0.load(Ordering::Relaxed), 0);
    }

    #[cfg(feature = "fallback-page")]
    #[tokio::test]
    async fn page_fallback_keeps_not_found_for_missing_assets() {
        let opts = fixture_settings("toml/handler.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        req_handler_opts.page_fallback = b"fallback".to_vec();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        // A browser asking for a favicon that does not exist.
        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = "http://localhost/favicon.ico".parse().unwrap();
        req.headers_mut().insert(
            "accept",
            HeaderValue::from_static("image/avif,image/webp,image/*,*/*;q=0.8"),
        );
        let resp = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("missing asset request should succeed");
        assert_eq!(resp.status(), 404);

        // A browser navigating to a client-side route.
        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = "http://localhost/missing-route".parse().unwrap();
        req.headers_mut()
            .insert("accept", HeaderValue::from_static("text/html,*/*;q=0.8"));
        let resp = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("fallback request should succeed");
        assert_eq!(resp.status(), 200);
    }

    #[cfg(all(feature = "basic-auth", feature = "mem-cache"))]
    #[tokio::test]
    async fn basic_auth_options_never_serves_cached_file() {
        use http_body_util::BodyExt;
        use static_web_server::cors;
        use static_web_server::mem_cache::cache;
        use static_web_server::settings::{Advanced, file::MemoryCache};

        let opts = fixture_settings("toml/handler.toml");
        let advanced = Advanced {
            memory_cache: Some(MemoryCache {
                capacity: None,
                ttl: None,
                tti: None,
                max_file_size: None,
            }),
            ..Default::default()
        };
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, Some(advanced));
        req_handler_opts.basic_auth =
            "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q".into();
        req_handler_opts.cors = cors::new("*", "", "");
        cache::init(&mut req_handler_opts).unwrap();
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let request = |method: Method, headers: &[(&'static str, &'static str)]| {
            let mut req = Request::new(());
            *req.method_mut() = method;
            *req.uri_mut() = "http://localhost/index.html".parse().unwrap();
            for (name, value) in headers {
                req.headers_mut()
                    .insert(*name, HeaderValue::from_static(value));
            }
            req
        };
        let auth = ("authorization", "Basic anE6anE=");

        // An authenticated client caches the file
        for _ in 0..2 {
            let mut req = request(Method::GET, &[auth]);
            let resp = req_handler.handle(&mut req, remote_addr).await.unwrap();
            assert_eq!(resp.status(), 200);
            resp.into_body().collect().await.unwrap();
        }
        let mut req = request(Method::GET, &[auth]);
        let resp = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(resp.headers()["x-cache"], "HIT");

        // A plain `OPTIONS` request must authenticate
        let mut req = request(Method::OPTIONS, &[]);
        let resp = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(resp.status(), 401);

        // A CORS preflight skips authentication but gets no file content
        let mut req = request(
            Method::OPTIONS,
            &[
                ("origin", "https://example.com"),
                ("access-control-request-method", "GET"),
            ],
        );
        let resp = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(resp.status(), 204);
        assert!(resp.headers().get("x-cache").is_none());
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert!(body.is_empty());
    }
}
