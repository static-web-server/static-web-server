#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(any(
    feature = "compression",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
    feature = "compression-deflate"
))]
#[cfg(test)]
pub mod tests {
    use headers::HeaderValue;
    use hyper::Request;
    use std::net::SocketAddr;

    #[cfg(any(feature = "compression", feature = "compression-deflate"))]
    use static_web_server::settings::CompressionLevel;
    use static_web_server::{
        settings::cli::General,
        testing::fixtures::{
            REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
        },
    };

    #[tokio::test]
    async fn compression_file() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression: true,
            compression_static: true,
            etag: true,
            index_files: "index.htm, index.html".to_owned(),
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html; charset=utf-8"))
                );
                assert_eq!(
                    res.headers().get("vary"),
                    Some(&HeaderValue::from_static("accept-encoding"))
                );
                assert_eq!(
                    res.headers().get("content-encoding"),
                    Some(&HeaderValue::from_static("br"))
                );
                assert_eq!(
                    res.headers().get("cache-control"),
                    Some(&HeaderValue::from_static("no-cache"))
                );
                assert_eq!(
                    res.headers().get("server"),
                    Some(&HeaderValue::from_static("Static Web Server"))
                );

                let vary_values = res
                    .headers()
                    .get("vary")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .rsplit(',')
                    .map(|f| f.trim())
                    .collect::<Vec<_>>();
                const EXPECTED: [&str; 1] = ["accept-encoding"];
                assert!(EXPECTED.iter().all(|s| vary_values.contains(s)));
            }
            Err(err) => panic!("unexpected error: {err}"),
        };
    }

    // The HTTP `deflate` coding is the zlib format (RFC 9110, section 8.4.1.2),
    // not a raw DEFLATE stream.
    #[cfg(any(feature = "compression", feature = "compression-deflate"))]
    async fn deflate_body(level: CompressionLevel) -> Vec<u8> {
        use http_body_util::BodyExt;

        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression: true,
            compression_static: false,
            compression_level: level,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/assets/index.html".parse().unwrap();
        req.headers_mut()
            .insert(http::header::ACCEPT_ENCODING, "deflate".parse().unwrap());

        let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-encoding"], "deflate");

        res.into_body().collect().await.unwrap().to_bytes().to_vec()
    }

    #[cfg(any(feature = "compression", feature = "compression-deflate"))]
    async fn zlib_decode(body: &[u8]) -> Vec<u8> {
        use async_compression::tokio::bufread::ZlibDecoder;
        use tokio::io::AsyncReadExt;

        let mut decoded = Vec::new();
        ZlibDecoder::new(body)
            .read_to_end(&mut decoded)
            .await
            .expect("deflate body must decode as zlib");
        decoded
    }

    #[cfg(any(feature = "compression", feature = "compression-deflate"))]
    #[tokio::test]
    async fn compression_deflate_uses_zlib_format() {
        let body = deflate_body(CompressionLevel::Default).await;
        // zlib header: CM = 8 (deflate), CINFO = 7 (32K window)
        assert_eq!(
            body.first(),
            Some(&0x78),
            "deflate body must start with a zlib header"
        );

        let expected = std::fs::read("tests/fixtures/public/assets/index.html").unwrap();
        assert_eq!(zlib_decode(&body).await, expected);
    }

    #[cfg(any(feature = "compression", feature = "compression-deflate"))]
    #[tokio::test]
    async fn compression_deflate_uses_zlib_format_at_every_level() {
        let expected = std::fs::read("tests/fixtures/public/assets/index.html").unwrap();

        for level in [
            CompressionLevel::Fastest,
            CompressionLevel::Default,
            CompressionLevel::Best,
        ] {
            let body = deflate_body(level).await;
            assert_eq!(
                body.first(),
                Some(&0x78),
                "deflate body must start with a zlib header ({level:?})"
            );
            // RFC 1950: CMF * 256 + FLG is a multiple of 31
            let header = body
                .get(..2)
                .map(|b| (u16::from(b[0]) << 8) | u16::from(b[1]));
            assert!(
                header.is_some_and(|h| h % 31 == 0),
                "deflate body must start with a valid zlib header ({level:?})"
            );
            assert_eq!(zlib_decode(&body).await, expected, "{level:?}");
        }
    }
}
