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
            index_files: "index.htm, index.html".to_owned(),
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
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
                    Some(&HeaderValue::from_static("max-age=86400"))
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

    // A `Content-Encoding` on a response without content describes nothing, and
    // a cache would merge it from a `304` into its stored identity response.
    #[tokio::test]
    async fn compression_skips_responses_without_content() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression: true,
            compression_static: false,
            redirect_trailing_slash: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/assets/index.html".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );
        let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key("content-encoding"));
        let last_modified = res.headers()["last-modified"].clone();

        let cases = [
            (
                "/assets/index.html",
                "if-modified-since",
                last_modified,
                304,
            ),
            (
                "/assets/index.html",
                "if-unmodified-since",
                HeaderValue::from_static("Mon, 01 Jan 1990 00:00:00 GMT"),
                412,
            ),
            (
                "/assets/index.html",
                "range",
                HeaderValue::from_static("bytes=99999-"),
                416,
            ),
            ("/assets", "accept", HeaderValue::from_static("*/*"), 308),
        ];
        for (path, name, value, status) in cases {
            for method in [hyper::Method::GET, hyper::Method::HEAD] {
                let mut req = Request::default();
                *req.method_mut() = method.clone();
                *req.uri_mut() = format!("http://localhost{path}").parse().unwrap();
                req.headers_mut().insert(
                    http::header::ACCEPT_ENCODING,
                    "gzip, deflate, br, zstd".parse().unwrap(),
                );
                req.headers_mut().insert(name, value.clone());

                let mut res = req_handler.handle(&mut req, remote_addr).await.unwrap();
                assert_eq!(res.status(), status, "{method} {path} {name}");
                assert_eq!(
                    res.headers().get("content-encoding"),
                    None,
                    "{method} {path} {name}"
                );
                let body = hyper::body::to_bytes(res.body_mut()).await.unwrap();
                assert!(body.is_empty(), "{method} {path} {name}");
            }
        }
    }

    // A `206 Partial Content` response carries a byte range of the identity
    // representation (its `Content-Range` counts identity bytes), so it must not
    // be compressed afterwards.
    #[tokio::test]
    async fn compression_skips_partial_content() {
        let codings = [
            #[cfg(any(feature = "compression", feature = "compression-deflate"))]
            "deflate",
            #[cfg(any(feature = "compression", feature = "compression-gzip"))]
            "gzip",
            #[cfg(any(feature = "compression", feature = "compression-brotli"))]
            "br",
            #[cfg(any(feature = "compression", feature = "compression-zstd"))]
            "zstd",
        ];
        let file = std::fs::read("tests/fixtures/public/assets/index.html").unwrap();

        for coding in codings {
            let opts = fixture_settings("toml/handler_fixtures.toml");
            let general = General {
                compression: true,
                compression_static: false,
                ..opts.general
            };
            let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
            let req_handler = fixture_req_handler(req_handler_opts);
            let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

            let mut req = Request::default();
            *req.method_mut() = hyper::Method::GET;
            *req.uri_mut() = "http://localhost/assets/index.html".parse().unwrap();
            req.headers_mut()
                .insert(http::header::ACCEPT_ENCODING, coding.parse().unwrap());
            req.headers_mut()
                .insert(http::header::RANGE, "bytes=0-399".parse().unwrap());

            let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
            assert_eq!(res.status(), 206, "{coding}");
            assert_eq!(
                res.headers().get("content-encoding"),
                None,
                "a 206 response must not be compressed ({coding})"
            );
            assert_eq!(
                res.headers()["content-range"],
                format!("bytes 0-399/{}", file.len()),
                "{coding}"
            );
            assert_eq!(res.headers()["content-length"], "400", "{coding}");

            let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
            assert_eq!(body, file[..400], "{coding}");
        }
    }
}
