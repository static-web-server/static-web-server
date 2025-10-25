#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use headers::{HeaderMap, HeaderValue};
    use hyper::{Method, Request};
    use std::net::SocketAddr;

    use static_web_server::cors;
    use static_web_server::http_ext::MethodExt;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    #[test]
    fn allow_methods() {
        let cors = cors::new("*", "", "").unwrap();
        let headers = HeaderMap::new();
        let methods = &[Method::GET, Method::HEAD, Method::OPTIONS];
        for method in methods {
            assert!(cors.check_request(method, &headers).is_ok());
        }

        let cors = cors::new("https://localhost", "", "").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        headers.insert("access-control-request-method", "GET".parse().unwrap());
        for method in methods {
            let res = cors.check_request(method, &headers);
            assert!(res.is_ok());

            let (res_headers, _) = res.unwrap();
            let allow_methods = res_headers
                .get("access-control-allow-methods")
                .unwrap()
                .to_str()
                .unwrap()
                .rsplit(',')
                .map(|f| f.trim())
                .collect::<Vec<_>>();

            const EXPECTED: [&str; 3] = ["GET", "OPTIONS", "HEAD"];
            EXPECTED.iter().all(|s| allow_methods.contains(s));
        }
    }

    #[test]
    fn disallow_methods() {
        let cors = cors::new("*", "", "").unwrap();
        let headers = HeaderMap::new();
        let methods = [
            Method::CONNECT,
            Method::DELETE,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ];
        for method in methods {
            let res = cors.check_request(&method, &headers);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert!(res.0.is_empty());
            assert!(matches!(res.1, cors::Validated::NotCors));
        }
    }

    #[test]
    fn origin_allowed() {
        let cors = cors::new("*", "", "").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        let methods = [Method::GET, Method::HEAD, Method::OPTIONS];
        for method in methods {
            let res = cors.check_request(&method, &headers);
            if method == Method::OPTIONS {
                // Forbidden (403) - preflight request missing access-control-request-method header
                assert!(res.is_err())
            } else {
                assert!(res.is_ok())
            }
        }
    }

    #[test]
    fn origin_not_allowed() {
        let cors = cors::new("https://localhost.rs", "", "").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        let methods = [Method::GET, Method::HEAD, Method::OPTIONS];
        for method in methods {
            let res = cors.check_request(&method, &headers);
            assert!(res.is_err());
            assert!(matches!(res.unwrap_err(), cors::Forbidden::Origin))
        }
    }

    #[test]
    fn method_allowed() {
        let cors = cors::new("*", "", "").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        headers.insert("access-control-request-method", "GET".parse().unwrap());
        let methods = [Method::GET, Method::HEAD, Method::OPTIONS];
        for method in methods {
            assert!(cors.check_request(&method, &headers).is_ok())
        }
    }

    #[test]
    fn method_disallowed() {
        let cors = cors::new("*", "", "").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        headers.insert("access-control-request-method", "POST".parse().unwrap());
        let methods = [Method::GET, Method::HEAD, Method::OPTIONS];
        for method in methods {
            let res = cors.check_request(&method, &headers);
            if method == Method::OPTIONS {
                // Forbidden (403) - preflight request missing access-control-request-method header
                assert!(res.is_err())
            } else {
                assert!(res.is_ok())
            }
        }
    }

    #[test]
    fn headers_allowed() {
        let cors = cors::new("*", "", "").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        headers.insert("access-control-request-method", "GET".parse().unwrap());
        headers.insert(
            "access-control-request-headers",
            "origin,content-type".parse().unwrap(),
        );
        let methods = [Method::OPTIONS];
        for method in methods {
            let res = cors.check_request(&method, &headers);
            assert!(res.is_ok())
        }
    }

    #[test]
    fn headers_invalid() {
        let cors = cors::new("*", "", "").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        headers.insert(
            "access-control-request-method",
            "GET,HEAD,OPTIONS".parse().unwrap(),
        );
        headers.insert(
            "access-control-request-headers",
            "origin, content-type".parse().unwrap(),
        );
        let methods = [Method::GET, Method::HEAD, Method::OPTIONS];
        for method in &methods {
            let res = cors.check_request(method, &headers);
            if method == Method::OPTIONS {
                assert!(res.is_err())
            } else {
                assert!(res.is_ok())
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        headers.insert("access-control-request-method", "GET".parse().unwrap());
        headers.insert(
            "access-control-request-headers",
            "origin,authorization".parse().unwrap(),
        );
        for method in methods {
            let res = cors.check_request(&method, &headers);
            if method == Method::OPTIONS {
                assert!(res.is_err())
            } else {
                assert!(res.is_ok())
            }
        }
    }

    #[tokio::test]
    async fn handler_allowed_methods() {
        let mut settings = fixture_settings("toml/handler.toml");
        let origin = "http://localhost".to_owned();
        settings.general.cors_allow_origins = origin.clone();

        let mut req_handler_opts = fixture_req_handler_opts(settings.general, settings.advanced);
        req_handler_opts.cors = cors::new("*", "", "");
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
            let mut req = Request::default();
            let mut headers = HeaderMap::new();
            headers.insert("origin", HeaderValue::from_str(origin.as_str()).unwrap());
            headers.insert(
                "access-control-request-method",
                "GET, HEAD, OPTIONS".parse().unwrap(),
            );
            *req.method_mut() = method.clone();
            *req.headers_mut() = headers;
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

                        let allow_methods = resp
                            .headers()
                            .get("access-control-allow-methods")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .rsplit(',')
                            .map(|f| f.trim())
                            .collect::<Vec<_>>();
                        const METHODS: [&str; 3] = ["GET", "OPTIONS", "HEAD"];
                        assert!(METHODS.iter().all(|s| allow_methods.contains(s)));

                        let vary_values = resp
                            .headers()
                            .get("vary")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .rsplit(',')
                            .map(|f| f.trim())
                            .collect::<Vec<_>>();

                        #[cfg(not(feature = "compression"))]
                        const EXPECTED: [&str; 1] = ["origin"];
                        #[cfg(feature = "compression")]
                        const EXPECTED: [&str; 2] = ["origin", "accept-encoding"];

                        assert!(EXPECTED.iter().all(|s| vary_values.contains(s)));
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
