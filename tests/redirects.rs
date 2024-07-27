#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use hyper::Request;
    use std::net::SocketAddr;

    use static_web_server::testing::fixtures::{
        fixture_req_handler, fixture_settings, REMOTE_ADDR,
    };

    #[tokio::test]
    async fn redirects_skipped() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html");
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_host() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://127.0.0.1:1234".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 301);
                assert_eq!(res.headers()["location"], "http://localhost:1234/");
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_glob_groups_1() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/assets/main.css".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 301);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-styles/style.css"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_glob_groups_2() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/style.css".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 301);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-styles/style.min.css"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_glob_groups_3() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/rust-lang.rs".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 302);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-languages/rust.lang.rs"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_glob_groups_4() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/assets/main.js".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 302);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-scripts/main.js"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_glob_groups_5() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/images/avatar.jpeg".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 302);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-images/images/avatar.jpeg"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_glob_groups_6() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/fonts/title.ttf".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 302);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-fonts/fonts/title.woff"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn redirects_glob_groups_generic_1() {
        let opts = fixture_settings("toml/redirects.toml");
        let req_handler = fixture_req_handler(opts.general, opts.advanced);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/generic-page.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 301);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-generic/generic-page.html"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }
}
