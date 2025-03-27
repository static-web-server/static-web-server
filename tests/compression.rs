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
            fixture_req_handler, fixture_req_handler_opts, fixture_settings, REMOTE_ADDR,
        },
    };

    #[tokio::test]
    async fn compression_file() {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let general = General {
            compression: true,
            compression_static: true,
            ..opts.general
        };
        let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/index.html".parse().unwrap();
        req.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );
                assert_eq!(
                    res.headers().get("vary"),
                    Some(&HeaderValue::from_static("accept-encoding"))
                );
                assert_eq!(
                    res.headers().get("content-encoding"),
                    Some(&HeaderValue::from_static("gzip"))
                );
                assert_eq!(
                    res.headers().get("cache-control"),
                    Some(&HeaderValue::from_static("public, max-age=86400"))
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
}
