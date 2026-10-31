#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for the `/livez` and `/readyz` health endpoints.

#[cfg(test)]
pub mod tests {
    use http::header::CONTENT_TYPE;
    use http_body_util::BodyExt;
    use hyper::{Method, Request, Response, StatusCode};
    use std::net::SocketAddr;

    use static_web_server::body::Body;
    use static_web_server::handler::RequestHandlerOpts;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    const PROBES: [&str; 2] = ["/livez", "/readyz"];

    async fn request(
        method: Method,
        uri: &str,
        configure: impl FnOnce(&mut RequestHandlerOpts),
    ) -> Response<Body> {
        let opts = fixture_settings("toml/handler_fixtures.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        configure(&mut req_handler_opts);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = method;
        *req.uri_mut() = ["http://localhost", uri].concat().parse().unwrap();

        req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler must succeed")
    }

    fn enable(opts: &mut RequestHandlerOpts) {
        opts.health = true;
    }

    async fn body_bytes(res: Response<Body>) -> Vec<u8> {
        res.into_body().collect().await.unwrap().to_bytes().to_vec()
    }

    #[tokio::test]
    async fn health_probes_return_ok() {
        for path in PROBES {
            let res = request(Method::GET, path, enable).await;
            assert_eq!(res.status(), StatusCode::OK, "{path}");
            assert_eq!(
                res.headers()[CONTENT_TYPE],
                "text/plain; charset=utf-8",
                "{path}"
            );
            assert_eq!(body_bytes(res).await, b"OK", "{path}");

            let res = request(Method::HEAD, path, enable).await;
            assert_eq!(res.status(), StatusCode::OK, "HEAD {path}");
            assert!(body_bytes(res).await.is_empty(), "HEAD {path}");
        }
    }

    #[tokio::test]
    async fn health_disabled_probes_fall_through_to_files() {
        for path in PROBES {
            let res = request(Method::GET, path, |opts| opts.health = false).await;
            assert_eq!(res.status(), StatusCode::NOT_FOUND, "{path}");
        }
    }

    #[tokio::test]
    async fn health_legacy_path_is_not_a_probe() {
        let res = request(Method::GET, "/health", enable).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[cfg(feature = "basic-auth")]
    #[tokio::test]
    async fn health_probes_skip_basic_auth() {
        for path in PROBES {
            let res = request(Method::GET, path, |opts| {
                opts.health = true;
                opts.basic_auth =
                    "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q".into();
            })
            .await;
            assert_eq!(res.status(), StatusCode::OK, "{path}");
        }
    }

    #[tokio::test]
    async fn health_probes_skip_maintenance_mode() {
        for path in PROBES {
            let res = request(Method::GET, path, |opts| {
                opts.health = true;
                opts.maintenance_mode = true;
            })
            .await;
            assert_eq!(res.status(), StatusCode::OK, "{path}");
        }
    }
}
