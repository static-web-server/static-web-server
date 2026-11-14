#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use hyper::{Body, Request, Response};
    use std::net::SocketAddr;

    use static_web_server::Settings;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts,
    };

    const COMMON_HEADERS: [(&str, &str); 3] = [
        ("x-frame-options", "DENY"),
        ("x-content-type-options", "nosniff"),
        ("content-security-policy", "frame-ancestors 'self'"),
    ];

    async fn get_index(args: &[&str]) -> Response<Body> {
        let mut argv = vec!["static-web-server", "--root", "tests/fixtures/public"];
        argv.extend_from_slice(args);
        let opts = Settings::get_unparsed(false, &argv).expect("settings must parse");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/index.htm".parse().unwrap();

        let res = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("unexpected error");
        assert_eq!(res.status(), 200);
        res
    }

    fn assert_common_headers(res: &Response<Body>, present: bool) {
        for (name, value) in COMMON_HEADERS {
            match res.headers().get(name) {
                Some(v) => {
                    assert!(present, "unexpected `{name}` header");
                    assert_eq!(v, value);
                }
                None => assert!(!present, "missing `{name}` header"),
            }
        }
    }

    #[tokio::test]
    async fn security_headers_disabled_by_default_on_plain_http() {
        let res = get_index(&[]).await;
        assert_common_headers(&res, false);
        assert!(!res.headers().contains_key("strict-transport-security"));
    }

    #[tokio::test]
    async fn plain_http_omits_hsts() {
        let res = get_index(&["--security-headers=true"]).await;
        assert_common_headers(&res, true);
        assert!(
            !res.headers().contains_key("strict-transport-security"),
            "HSTS must not be sent over plain HTTP"
        );
    }

    #[cfg(feature = "http2")]
    const TLS_ARGS: [&str; 5] = [
        "--http2",
        "--http2-tls-cert",
        "tests/tls/local.dev_cert.pkcs8.pem",
        "--http2-tls-key",
        "tests/tls/local.dev_key.pkcs8.pem",
    ];

    #[cfg(feature = "http2")]
    #[tokio::test]
    async fn tls_sends_hsts_by_default() {
        for extra in [&[][..], &["--security-headers=true"][..]] {
            let mut args = TLS_ARGS.to_vec();
            args.extend_from_slice(extra);
            let res = get_index(&args).await;
            assert_common_headers(&res, true);
            assert_eq!(
                res.headers().get("strict-transport-security").unwrap(),
                "max-age=63072000; includeSubDomains; preload",
                "args: {args:?}"
            );
        }
    }

    #[cfg(feature = "http2")]
    #[tokio::test]
    async fn tls_security_headers_false_disables_them() {
        let mut args = TLS_ARGS.to_vec();
        args.push("--security-headers=false");
        let res = get_index(&args).await;
        assert_common_headers(&res, false);
        assert!(!res.headers().contains_key("strict-transport-security"));
    }
}
