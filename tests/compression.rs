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

    // A HEAD response carries the header fields of the GET response
    // (RFC 9110, section 9.3.2), including `Content-Encoding`.
    #[tokio::test]
    async fn compression_head_matches_get_headers() {
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

            let mut responses = Vec::new();
            for method in [hyper::Method::GET, hyper::Method::HEAD] {
                let mut req = Request::new(());
                *req.method_mut() = method;
                *req.uri_mut() = "http://localhost/assets/index.html".parse().unwrap();
                req.headers_mut()
                    .insert(http::header::ACCEPT_ENCODING, coding.parse().unwrap());
                let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
                assert_eq!(res.status(), 200, "{coding}");
                responses.push(res);
            }
            let (get, head) = (&responses[0], &responses[1]);

            assert_eq!(
                get.headers().get("content-encoding"),
                Some(&HeaderValue::from_static(coding))
            );
            assert_eq!(
                head.headers().get("content-encoding"),
                get.headers().get("content-encoding"),
                "HEAD must send the content-encoding of GET ({coding})"
            );
            assert_eq!(
                head.headers().get("vary"),
                get.headers().get("vary"),
                "{coding}"
            );
            assert_eq!(
                head.headers().get("content-length"),
                None,
                "HEAD must not send the identity content-length ({coding})"
            );
        }
    }
}
