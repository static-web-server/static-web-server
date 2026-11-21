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
    use std::path::Path;

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

    async fn body(resp: Response<Body>) -> Vec<u8> {
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec()
    }

    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(Path::new("tests/fixtures/images").join(name)).unwrap()
    }

    /// Whether the bytes start with the file signature of the image type.
    fn has_signature(bytes: &[u8], content_type: &str) -> bool {
        match content_type {
            "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
            "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "image/gif" => bytes.starts_with(b"GIF8"),
            "image/webp" => bytes.get(..4) == Some(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
            "image/avif" => bytes.get(4..12) == Some(b"ftypavif"),
            _ => false,
        }
    }

    /// Asserts a `200` response serves the image fixture byte for byte, with
    /// the content type its signature matches.
    async fn assert_serves(resp: Response<Body>, name: &str, content_type: &str) {
        let expected = fixture(name);
        assert_eq!(resp.status(), 200, "{name}");
        assert_eq!(header(&resp, "content-type"), Some(content_type), "{name}");
        let len = expected.len().to_string();
        assert_eq!(
            header(&resp, "content-length"),
            Some(len.as_str()),
            "{name}"
        );
        let bytes = body(resp).await;
        assert!(
            has_signature(&bytes, content_type),
            "{name} is not {content_type}"
        );
        assert!(bytes == expected, "{name} body differs from the fixture");
    }

    #[tokio::test]
    async fn image_negotiation_serves_avif_to_chrome() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/sws.jpg", CHROME).await;

        assert_eq!(header(&resp, "cache-control"), Some("max-age=31536000"));
        assert!(varies_on_accept(&resp));
        assert_serves(resp, "sws.jpg.avif", "image/avif").await;
    }

    #[tokio::test]
    async fn image_negotiation_serves_webp_when_avif_not_accepted() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/sws.jpg", "image/webp,*/*").await;

        assert!(varies_on_accept(&resp));
        assert_serves(resp, "sws.jpg.webp", "image/webp").await;
    }

    #[tokio::test]
    async fn image_negotiation_serves_next_variant_on_disk() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/sws.png", CHROME).await;

        assert!(varies_on_accept(&resp));
        assert_serves(resp, "sws.png.webp", "image/webp").await;
    }

    #[tokio::test]
    async fn image_negotiation_serves_original_to_other_clients() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        for accept in ["*/*", "image/*", "image/jpeg, image/webp;q=0.5"] {
            let resp = get(&handler, "/sws.jpg", accept).await;

            assert!(varies_on_accept(&resp), "{accept}");
            assert_serves(resp, "sws.jpg", "image/jpeg").await;
        }

        let resp = send(&handler, Method::GET, "/sws.jpg", &[]).await;
        assert!(varies_on_accept(&resp));
        assert_serves(resp, "sws.jpg", "image/jpeg").await;
    }

    #[tokio::test]
    async fn image_negotiation_serves_original_without_variants() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = get(&handler, "/sws.gif", CHROME).await;

        assert!(varies_on_accept(&resp));
        assert_serves(resp, "sws.gif", "image/gif").await;
    }

    #[tokio::test]
    async fn image_negotiation_disabled_serves_original() {
        let handler = handler("toml/image_negotiation_disabled.toml");
        let resp = get(&handler, "/sws.jpg", CHROME).await;

        assert!(!varies_on_accept(&resp));
        assert_serves(resp, "sws.jpg", "image/jpeg").await;
    }

    #[tokio::test]
    async fn image_negotiation_head_matches_get() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = send(
            &handler,
            Method::HEAD,
            "/sws.jpg",
            &[(hyper::header::ACCEPT, CHROME)],
        )
        .await;

        let len = fixture("sws.jpg.avif").len().to_string();
        assert_eq!(resp.status(), 200);
        assert_eq!(header(&resp, "content-type"), Some("image/avif"));
        assert_eq!(header(&resp, "content-length"), Some(len.as_str()));
        assert!(varies_on_accept(&resp));
    }

    #[tokio::test]
    async fn image_negotiation_options_does_not_vary() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = send(
            &handler,
            Method::OPTIONS,
            "/sws.jpg",
            &[(hyper::header::ACCEPT, CHROME)],
        )
        .await;

        assert_eq!(resp.status(), 204);
        assert!(!varies_on_accept(&resp));
    }

    #[tokio::test]
    async fn image_negotiation_etag_belongs_to_variant() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let variant = get(&handler, "/sws.jpg", CHROME).await;
        let original = get(&handler, "/sws.jpg", "*/*").await;
        let variant_etag = header(&variant, "etag").unwrap().to_owned();
        let original_etag = header(&original, "etag").unwrap().to_owned();
        assert_ne!(variant_etag, original_etag);

        let variant_etag = HeaderValue::from_str(&variant_etag).unwrap();
        let mut req = Request::new(());
        *req.uri_mut() = "http://localhost/sws.jpg".parse().unwrap();
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
        assert_serves(resp, "sws.jpg", "image/jpeg").await;
    }

    #[tokio::test]
    async fn image_negotiation_range_applies_to_variant() {
        let handler = handler("toml/image_negotiation_enabled.toml");
        let resp = send(
            &handler,
            Method::GET,
            "/sws.jpg",
            &[
                (hyper::header::ACCEPT, CHROME),
                (hyper::header::RANGE, "bytes=0-11"),
            ],
        )
        .await;

        let avif = fixture("sws.jpg.avif");
        let content_range = format!("bytes 0-11/{}", avif.len());
        assert_eq!(resp.status(), 206);
        assert_eq!(header(&resp, "content-range"), Some(content_range.as_str()));
        assert!(varies_on_accept(&resp));
        assert_eq!(body(resp).await, &avif[..12]);
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
        std::fs::write(root.join("sws.png"), fixture("sws.png")).unwrap();
        std::fs::write(root.join("target.webp"), fixture("sws.png.webp")).unwrap();
        std::os::unix::fs::symlink(root.join("target.webp"), root.join("sws.png.webp")).unwrap();

        let opts = fixture_settings("toml/image_negotiation_enabled.toml");
        let general = static_web_server::settings::cli::General {
            root,
            follow_symlinks: false,
            ..opts.general
        };
        let handler = fixture_req_handler(fixture_req_handler_opts(general, opts.advanced));
        let resp = get(&handler, "/sws.png", CHROME).await;

        assert!(varies_on_accept(&resp));
        assert_serves(resp, "sws.png", "image/png").await;
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
            for (accept, name, content_type) in [
                (CHROME, "sws.jpg.avif", "image/avif"),
                ("*/*", "sws.jpg", "image/jpeg"),
                ("image/webp", "sws.jpg.webp", "image/webp"),
            ] {
                let resp = get(&handler, "/sws.jpg", accept).await;
                assert!(varies_on_accept(&resp), "{accept}");
                assert_serves(resp, name, content_type).await;
            }
        }
    }
}
