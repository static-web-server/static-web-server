#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

pub mod tests {
    use hyper::Request;
    use std::net::SocketAddr;

    use static_web_server::testing::fixtures::{fixture_req_handler, REMOTE_ADDR};

    #[tokio::test]
    async fn redirects_default() {
        let req_handler = fixture_req_handler("toml/redirects.toml");
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost:1234/assets/favicon.ico".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 302);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost:1234/files/assets/favicon.ico"
                );
            }
            Err(status) => {
                panic!("expected a status 302 but got {status}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_host() {
        let req_handler = fixture_req_handler("toml/redirects.toml");
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://127.0.0.1:1234".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 301);
                assert_eq!(res.headers()["location"], "http://localhost:1234/");
            }
            Err(status) => {
                panic!("expected a status 301 but got {status}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_skipped() {
        let req_handler = fixture_req_handler("toml/redirects.toml");
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost:1234".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html");
            }
            Err(status) => {
                panic!("expected a status 200 but got {status}")
            }
        };
    }
}
