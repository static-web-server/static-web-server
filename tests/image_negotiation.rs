#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use headers::HeaderValue;
    use http_body_util::BodyExt;
    use hyper::{Method, Request, Response};
    use std::net::SocketAddr;

    use static_web_server::body::Body;
    use static_web_server::handler::RequestHandler;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    /// `Accept` sent by Chrome and Edge for images.
    const CHROME: &str = "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8";

    fn handler(toml: &str) -> RequestHandler {
        let opts = fixture_settings(toml);
        fixture_req_handler(fixture_req_handler_opts(opts.general, opts.advanced))
    }

    async fn send(
        handler: &RequestHandler,
        method: Method,
        uri: &str,
        headers: &[(hyper::header::HeaderName, &'static str)],
    ) -> Response<Body> {
        let mut req = Request::new(());
        *req.method_mut() = method;
        *req.uri_mut() = format!("http://localhost{uri}").parse().unwrap();
        for (name, value) in headers {
            req.headers_mut()
                .insert(name, HeaderValue::from_static(value));
        }
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        handler.handle(&mut req, remote_addr).await.unwrap()
    }

    async fn get(handler: &RequestHandler, uri: &str, accept: &'static str) -> Response<Body> {
        send(
            handler,
            Method::GET,
            uri,
            &[(hyper::header::ACCEPT, accept)],
        )
        .await
    }

    fn header<'a>(resp: &'a Response<Body>, name: &str) -> Option<&'a str> {
        resp.headers().get(name).map(|v| v.to_str().unwrap())
    }

    /// Whether the response `Vary` header lists `accept` as a whole field name.
    fn varies_on_accept(resp: &Response<Body>) -> bool {
        header(resp, "vary").is_some_and(|v| v.split(',').any(|n| n.trim() == "accept"))
    }

    async fn body(resp: Response<Body>) -> String {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn image_negotiation_serves_avif_to_chrome() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/photo.jpg", CHROME).await;

        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/avif"));
        assert_eq!(header(&resp, "content-length"), Some("12"));
        assert_eq!(header(&resp, "cache-control"), Some("max-age=31536000"));
        assert!(varies_on_accept(&resp));
        assert_eq!(body(resp).await, "avif-variant");
    }

    #[tokio::test]
    async fn image_negotiation_serves_webp_when_avif_not_accepted() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/photo.jpg", "image/webp,*/*").await;

        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/webp"));
        assert!(varies_on_accept(&resp));
        assert_eq!(body(resp).await, "webp-variant");
    }

    #[tokio::test]
    async fn image_negotiation_serves_next_variant_on_disk() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/logo.png", CHROME).await;

        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/webp"));
        assert_eq!(body(resp).await, "webp");
    }

    #[tokio::test]
    async fn image_negotiation_serves_original_to_other_clients() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        for accept in ["*/*", "image/*", "image/jpeg, image/webp;q=0.5"] {
            let resp = get(&handler, "/photo.jpg", accept).await;

            assert_eq!(resp.status(), 200, "{accept}");
            assert_eq!(
                header(&resp, "content-type"),
                Some("image/jpeg"),
                "{accept}"
            );
            assert!(varies_on_accept(&resp), "{accept}");
            assert_eq!(body(resp).await, "jpeg", "{accept}");
        }

        let resp = send(&handler, Method::GET, "/photo.jpg", &[]).await;
        assert_eq!(header(&resp, "content-type"), Some("image/jpeg"));
        assert!(varies_on_accept(&resp));
    }

    #[tokio::test]
    async fn image_negotiation_serves_original_without_variants() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/plain.gif", CHROME).await;

        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/gif"));
        assert!(varies_on_accept(&resp));
        assert_eq!(body(resp).await, "gif");
    }

    #[tokio::test]
    async fn image_negotiation_disabled_serves_original() {
        let handler = handler("toml/image_negotiation_disabled.toml");
        let resp = get(&handler, "/photo.jpg", CHROME).await;

        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/jpeg"));
        assert!(!varies_on_accept(&resp));
        assert_eq!(body(resp).await, "jpeg");
    }

    #[tokio::test]
    async fn image_negotiation_head_matches_get() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = send(
            &handler,
            Method::HEAD,
            "/photo.jpg",
            &[(hyper::header::ACCEPT, CHROME)],
        )
        .await;

        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/avif"));
        assert_eq!(header(&resp, "content-length"), Some("12"));
        assert!(varies_on_accept(&resp));
    }

    #[tokio::test]
    async fn image_negotiation_options_does_not_vary() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = send(
            &handler,
            Method::OPTIONS,
            "/photo.jpg",
            &[(hyper::header::ACCEPT, CHROME)],
        )
        .await;

        assert_eq!(resp.status(), 204);
        assert!(!varies_on_accept(&resp));
    }

    #[tokio::test]
    async fn image_negotiation_etag_belongs_to_variant() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let variant = get(&handler, "/photo.jpg", CHROME).await;
        let original = get(&handler, "/photo.jpg", "*/*").await;
        let variant_etag = header(&variant, "etag").unwrap().to_owned();
        let original_etag = header(&original, "etag").unwrap().to_owned();
        assert_ne!(variant_etag, original_etag);

        let variant_etag = HeaderValue::from_str(&variant_etag).unwrap();
        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/photo.jpg".parse().unwrap();
        req.headers_mut()
            .insert(hyper::header::ACCEPT, HeaderValue::from_static(CHROME));
        req.headers_mut()
            .insert(hyper::header::IF_NONE_MATCH, variant_etag.clone());
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        let resp = handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(resp.status(), 304);
        assert!(varies_on_accept(&resp));

        // The variant's validator doesn't match the original
        *req.headers_mut().get_mut(hyper::header::ACCEPT).unwrap() =
            HeaderValue::from_static("*/*");
        let resp = handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(body(resp).await, "jpeg");
    }

    #[tokio::test]
    async fn image_negotiation_range_applies_to_variant() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = send(
            &handler,
            Method::GET,
            "/photo.jpg",
            &[
                (hyper::header::ACCEPT, CHROME),
                (hyper::header::RANGE, "bytes=0-3"),
            ],
        )
        .await;

        assert_eq!(resp.status(), 206);
        assert_eq!(header(&resp, "content-range"), Some("bytes 0-3/12"));
        assert!(varies_on_accept(&resp));
        assert_eq!(body(resp).await, "avif");
    }

    #[tokio::test]
    async fn image_negotiation_missing_original_is_not_found() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/missing.jpg", CHROME).await;
        assert_eq!(resp.status(), 404);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn image_negotiation_unservable_variant_falls_back_to_original() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        std::fs::write(root.join("target.webp"), "webp").unwrap();
        std::fs::write(root.join("link.png"), "png").unwrap();
        std::os::unix::fs::symlink(root.join("target.webp"), root.join("link.png.webp")).unwrap();

        let opts = fixture_settings("toml/image_negotiation_enabled.toml");
        let general = static_web_server::settings::cli::General {
            root,
            follow_symlinks: false,
            ..opts.general
        };
        let handler = fixture_req_handler(fixture_req_handler_opts(general, opts.advanced));
        let resp = get(&handler, "/link.png", CHROME).await;

        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/png"));
        assert!(varies_on_accept(&resp));
        assert_eq!(body(resp).await, "png");
    }

    #[cfg(feature = "mem-cache")]
    #[tokio::test]
    async fn image_negotiation_mem_cache_keeps_formats_apart() {
        let opts = fixture_settings("toml/image_negotiation_mem_cache.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        static_web_server::mem_cache::cache::init(&mut req_handler_opts).unwrap();
        let handler = fixture_req_handler(req_handler_opts);

        // Twice each, so the second round is served from the cache
        for _ in 0..2 {
            for (accept, content_type, expected) in [
                (CHROME, "image/avif", "avif-variant"),
                ("*/*", "image/jpeg", "jpeg"),
                ("image/webp", "image/webp", "webp-variant"),
            ] {
                let resp = get(&handler, "/photo.jpg", accept).await;
                assert_eq!(resp.status(), 200, "{accept}");
                assert_eq!(
                    header(&resp, "content-type"),
                    Some(content_type),
                    "{accept}"
                );
                assert!(varies_on_accept(&resp), "{accept}");
                assert_eq!(body(resp).await, expected, "{accept}");
            }
        }
    }
}
