#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use headers::HeaderValue;
    use hyper::{Method, Request};
    use std::net::SocketAddr;

    use static_web_server::http_ext::MethodExt;
    use static_web_server::testing::fixtures::{
        fixture_req_handler, fixture_settings, REMOTE_ADDR,
    };

    #[tokio::test]
    async fn custom_headers_apply_for_dir() {
        let opts = fixture_settings("toml/handler.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-deflate",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd"
                ))]
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
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/assets/index.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(
                    res.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/html"))
                );
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-deflate",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd"
                ))]
                assert_eq!(
                    res.headers().get("vary"),
                    Some(&HeaderValue::from_static("accept-encoding"))
                );
                assert_eq!(
                    res.headers().get("cache-control"),
                    Some(&HeaderValue::from_static("public, max-age=86400"))
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
        let settings = fixture_settings("toml/handler.toml");
        let req_handler = fixture_req_handler(settings.general, settings.advanced);
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
            let mut req = Request::default();
            *req.method_mut() = method.clone();
            *req.uri_mut() = "http://localhost/assets/index.html".parse().unwrap();

            match req_handler.handle(&mut req, remote_addr).await {
                Ok(resp) => {
                    if method.is_allowed() {
                        assert_eq!(resp.status(), 200);
                        assert_eq!(
                            resp.headers().get("content-type"),
                            Some(&HeaderValue::from_static("text/html"))
                        );
                        assert_eq!(
                            resp.headers().get("server"),
                            Some(&HeaderValue::from_static("Static Web Server"))
                        );
                    } else {
                        assert_eq!(resp.status(), 405);
                    }
                }
                Err(err) => {
                    panic!("unexpected error: {err}")
                }
            };
        }
    }
}
