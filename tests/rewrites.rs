#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
pub mod tests {
    use hyper::Request;
    use std::net::SocketAddr;

    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    #[tokio::test]
    async fn rewrites_skipped() {
        let mut opts = fixture_settings("toml/rewrites.toml");
        opts.general.index_files = "index.htm".to_owned();
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://development".parse().unwrap();

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
    async fn rewrites_glob_groups_1() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/some/error-page.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(mut res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html");

                let body = hyper::body::to_bytes(res.body_mut())
                    .await
                    .expect("unexpected bytes error during `body` conversion");
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("404 Content"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_2() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/error-page/50x.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(mut res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html");

                let body = hyper::body::to_bytes(res.body_mut())
                    .await
                    .expect("unexpected bytes error during `body` conversion");
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("50x Service Unavailable"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_3() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/errors/50x.html".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(mut res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/html");

                let body = hyper::body::to_bytes(res.body_mut())
                    .await
                    .expect("unexpected bytes error during `body` conversion");
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("50x Service Unavailable"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_4() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/scripts/main.js".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(mut res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/javascript");

                let body = hyper::body::to_bytes(res.body_mut())
                    .await
                    .expect("unexpected bytes error during `body` conversion");
                let body_str = std::str::from_utf8(&body).unwrap();
                assert!(body_str.contains("Static Web Server"))
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_5() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/images/icon.ico".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 302);
                assert_eq!(res.headers()["location"], "/assets/favicon.ico");
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_6() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.uri_mut() = "http://localhost/old/fonts/text.ttf".parse().unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 302);
                assert_eq!(
                    res.headers()["location"],
                    "http://localhost/new-fonts/text.woff"
                );
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }

    #[tokio::test]
    async fn rewrites_glob_groups_generic_1() {
        let opts = fixture_settings("toml/rewrites.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
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
