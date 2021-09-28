#![deny(warnings)]
#![deny(rust_2018_idioms)]

#[cfg(test)]
mod tests {
    use headers::HeaderMap;
    use http::Method;
    use static_web_server::cors;

    #[tokio::test]
    async fn allow_methods() {
        let cors = cors::new("*".to_owned()).unwrap();
        let headers = HeaderMap::new();
        let methods = &[Method::GET, Method::HEAD];
        for method in methods {
            assert!(cors.check_request(method, &headers).is_ok())
        }

        let cors = cors::new("https://localhost".to_owned()).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        for method in methods {
            assert!(cors.check_request(method, &headers).is_ok())
        }
    }

    #[test]
    fn disallow_methods() {
        let cors = cors::new("*".to_owned()).unwrap();
        let headers = HeaderMap::new();
        let methods = [
            Method::CONNECT,
            Method::DELETE,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ];
        for method in methods {
            let res = cors.check_request(&method, &headers);
            assert!(res.is_ok());
            assert!(matches!(res.unwrap(), cors::Validated::NotCors));
        }
    }

    #[tokio::test]
    async fn origin_allowed() {
        let cors = cors::new("*".to_owned()).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        let methods = [Method::GET, Method::HEAD];
        for method in methods {
            assert!(cors.check_request(&method, &headers).is_ok())
        }
    }

    #[tokio::test]
    async fn origin_not_allowed() {
        let cors = cors::new("https://localhost.rs".to_owned()).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://localhost".parse().unwrap());
        let methods = [Method::GET, Method::HEAD];
        for method in methods {
            let res = cors.check_request(&method, &headers);
            assert!(res.is_err());
            assert!(matches!(res.unwrap_err(), cors::Forbidden::Origin))
        }
    }
}
