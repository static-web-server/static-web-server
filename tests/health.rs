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
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    /// Send `GET <path>` with the given `Host` to a running server and return the raw response.
    async fn fetch(port: u16, path: &str, host: &str) -> String {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect");
        let req = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).await.expect("write");
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.expect("read");
        String::from_utf8_lossy(&buf).into_owned()
    }

    /// Send `GET <path>` to a running server and return the response status code.
    async fn status_of(port: u16, path: &str) -> u16 {
        fetch(port, path, "localhost")
            .await
            .split(' ')
            .nth(1)
            .and_then(|s| s.parse().ok())
            .expect("status code")
    }

    /// Run a server with the given extra arguments on a free port.
    fn spawn_server(
        extra: &[&str],
    ) -> (
        u16,
        tokio::sync::watch::Sender<()>,
        std::thread::JoinHandle<static_web_server::Result>,
    ) {
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("local addr").port();
        let mut args: Vec<String> = [
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--health",
        ]
        .map(String::from)
        .to_vec();
        args.extend(extra.iter().map(|s| s.to_string()));
        let server = std::thread::spawn(move || {
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let settings =
                static_web_server::Settings::get_unparsed(false, &refs).expect("settings");
            static_web_server::Server::new(settings)
                .expect("server")
                .with_pre_bound_listener(listener)
                .run_server_on_rt(Some(cancel_rx), || {}, false)
        });
        (port, cancel_tx, server)
    }

    /// Stop the server and wait for it to exit.
    async fn stop_server(
        cancel_tx: tokio::sync::watch::Sender<()>,
        server: std::thread::JoinHandle<static_web_server::Result>,
    ) {
        // The receiver is gone if shutdown already started.
        let _ = cancel_tx.send(());
        tokio::task::spawn_blocking(move || server.join().expect("server thread"))
            .await
            .expect("join")
            .expect("server result");
    }

    #[tokio::test]
    async fn health_readyz_fails_during_grace_period() {
        let (port, cancel_tx, server) = spawn_server(&["--grace-period", "3"]);

        assert_eq!(status_of(port, "/readyz").await, 200);
        cancel_tx.send(()).expect("cancel");

        let mut readyz = 200;
        for _ in 0..40 {
            readyz = status_of(port, "/readyz").await;
            if readyz == 503 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        assert_eq!(readyz, 503, "/readyz during grace period");
        assert_eq!(
            status_of(port, "/livez").await,
            200,
            "/livez during grace period"
        );

        stop_server(cancel_tx, server).await;
    }

    #[cfg(feature = "metrics")]
    #[tokio::test]
    async fn health_probes_skip_metrics() {
        let (port, cancel_tx, server) = spawn_server(&["--metrics"]);

        for path in PROBES {
            assert!(
                fetch(port, path, "probe.test")
                    .await
                    .starts_with("HTTP/1.1 200")
            );
        }
        assert!(
            fetch(port, "/index.htm", "file.test")
                .await
                .starts_with("HTTP/1.1 200")
        );

        let metrics = fetch(port, "/metrics", "localhost").await;
        assert!(metrics.contains(r#"host="file.test""#), "{metrics}");
        assert!(!metrics.contains(r#"host="probe.test""#), "{metrics}");

        stop_server(cancel_tx, server).await;
    }
}
