#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use headers::HeaderMap;
    use http::Method;
    use static_web_server::cors;

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
            assert!(cors.check_request(method, &headers).is_ok());
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
}
