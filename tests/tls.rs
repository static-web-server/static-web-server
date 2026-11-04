#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

// Settings validation tests

#[cfg(test)]
mod settings_tests {
    #[cfg(feature = "tls")]
    const CERT: &str = "tests/tls/local.dev_cert.pkcs8.pem";
    #[cfg(feature = "tls")]
    const KEY: &str = "tests/tls/local.dev_key.pkcs8.pem";

    /// `--tls`, `--tls-cert` and `--tls-key` must parse successfully and be
    /// reflected in the resolved `General` settings.
    #[cfg(feature = "tls")]
    #[test]
    fn tls_flags_parse_correctly() {
        use std::path::PathBuf;

        let settings = static_web_server::Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "tests/fixtures/public",
                "--port",
                "0",
                "--tls",
                "--tls-cert",
                CERT,
                "--tls-key",
                KEY,
            ],
        )
        .expect("settings with --tls must parse");

        assert!(settings.general.tls, "general.tls should be true");
        assert_eq!(
            settings.general.tls_cert,
            Some(PathBuf::from(CERT)),
            "tls_cert should be set"
        );
        assert_eq!(
            settings.general.tls_key,
            Some(PathBuf::from(KEY)),
            "tls_key should be set"
        );
    }

    /// Builds with the `tls` feature enable security headers by default,
    /// with or without `--tls`. Only HSTS depends on `--tls`.
    #[cfg(feature = "tls")]
    #[test]
    fn tls_build_enables_security_headers_by_default() {
        let plain = ["static-web-server", "--root", "tests/fixtures/public"];
        let tls = [
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--tls",
            "--tls-cert",
            CERT,
            "--tls-key",
            KEY,
        ];
        for args in [&plain[..], &tls[..]] {
            let settings = static_web_server::Settings::get_unparsed(false, args)
                .expect("settings must parse");
            assert!(
                settings.general.security_headers,
                "security headers should be on by default: {args:?}"
            );
        }
    }

    /// An explicit `--security-headers=false` must win over the TLS default,
    /// also when a config file without `security-headers` is loaded.
    #[cfg(feature = "tls")]
    #[test]
    fn tls_respects_security_headers_false() {
        for config in [None, Some("tests/fixtures/toml/handler.toml")] {
            let mut args = vec![
                "static-web-server",
                "--root",
                "tests/fixtures/public",
                "--port",
                "0",
                "--tls",
                "--tls-cert",
                CERT,
                "--tls-key",
                KEY,
                "--security-headers=false",
            ];
            if let Some(path) = config {
                args.extend(["--config-file", path]);
            }
            let settings = static_web_server::Settings::get_unparsed(false, &args)
                .expect("settings with --tls must parse");

            assert!(
                !settings.general.security_headers,
                "--security-headers=false was overridden (config file: {config:?})"
            );
        }
    }

    /// `--https-redirect` requires `--tls`; without it the settings parsing
    /// must return an error containing the expected diagnostic message.
    #[cfg(feature = "tls")]
    #[test]
    fn https_redirect_without_tls_fails() {
        match static_web_server::Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "tests/fixtures/public",
                "--port",
                "0",
                "--https-redirect",
            ],
        ) {
            Ok(_) => panic!("--https-redirect without --tls should have failed"),
            Err(err) => assert!(
                err.to_string().contains("--https-redirect requires TLS"),
                "unexpected error message: {err}"
            ),
        }
    }

    /// `--http2` requires `--tls`; without it the settings parsing must fail.
    #[cfg(all(feature = "tls", feature = "http2"))]
    #[test]
    fn http2_without_tls_fails() {
        match static_web_server::Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "tests/fixtures/public",
                "--port",
                "0",
                "--http2",
            ],
        ) {
            Ok(_) => panic!("--http2 without --tls should have failed"),
            Err(err) => assert!(
                err.to_string().contains("HTTP/2 requires TLS"),
                "unexpected error message: {err}"
            ),
        }
    }

    /// `--http2` combined with `--tls` must parse successfully and both flags
    /// must be reflected in the resolved settings.
    #[cfg(all(feature = "tls", feature = "http2"))]
    #[test]
    fn http2_with_tls_parses_correctly() {
        let settings = static_web_server::Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "tests/fixtures/public",
                "--port",
                "0",
                "--tls",
                "--tls-cert",
                CERT,
                "--tls-key",
                KEY,
                "--http2",
            ],
        )
        .expect("settings with --tls --http2 must parse");

        assert!(settings.general.tls, "general.tls should be true");
        assert!(settings.general.http2, "general.http2 should be true");
    }
}

// Live server integration tests
//
// Each test spawns a real SWS server in a dedicated thread (with its own tokio
// runtime) and connects to it using a TLS client. To avoid flakiness:
//
// 1. Ports are allocated by the OS via binding to port 0.
// 2. Readiness is determined by polling TCP connect (not sleeping).
// 3. Every async operation is wrapped in `tokio::time::timeout`.
// 4. The server is shut down via a watch channel, and `server.join()` is
//    performed in a blocking spawn to avoid starving the test runtime.

#[cfg(feature = "tls")]
mod test_helpers {
    use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
    use std::sync::Arc;
    use tokio_rustls::rustls::{
        ClientConfig, DigitallySignedStruct, Error as TlsError, SignatureScheme,
        client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    };

    /// Maximum time any single test operation is allowed to take.
    pub const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

    /// Grab a free local port.
    pub fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind to port 0")
            .local_addr()
            .unwrap()
            .port()
    }

    /// Open a TLS connection to the server on `port`, offering the `alpn` protocols.
    pub async fn tls_connect(
        port: u16,
        alpn: &[&str],
    ) -> tokio_rustls::client::TlsStream<tokio::net::TcpStream> {
        let connector = tokio_rustls::TlsConnector::from(tls_client_config(alpn));
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        let domain = ServerName::try_from("localhost").unwrap().to_owned();
        connector
            .connect(domain, stream)
            .await
            .expect("TLS handshake")
    }

    /// Wait until the server is accepting TCP connections on `port`.
    pub async fn wait_for_server(port: u16) {
        for _ in 0..100 {
            if tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_ok()
            {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        panic!("server did not become ready on port {port} within 10s");
    }

    /// Build a rustls `ClientConfig` that skips certificate verification (safe
    /// for tests where we control both ends) and advertises the given ALPN.
    pub fn tls_client_config(alpn: &[&str]) -> Arc<ClientConfig> {
        #[derive(Debug)]
        struct NoVerify;

        impl ServerCertVerifier for NoVerify {
            fn verify_server_cert(
                &self,
                _end_entity: &CertificateDer<'_>,
                _intermediates: &[CertificateDer<'_>],
                _server_name: &ServerName<'_>,
                _ocsp_response: &[u8],
                _now: UnixTime,
            ) -> Result<ServerCertVerified, TlsError> {
                Ok(ServerCertVerified::assertion())
            }

            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, TlsError> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, TlsError> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
                vec![
                    SignatureScheme::RSA_PKCS1_SHA256,
                    SignatureScheme::RSA_PKCS1_SHA384,
                    SignatureScheme::RSA_PKCS1_SHA512,
                    SignatureScheme::ECDSA_NISTP256_SHA256,
                    SignatureScheme::ECDSA_NISTP384_SHA384,
                    SignatureScheme::ECDSA_NISTP521_SHA512,
                    SignatureScheme::RSA_PSS_SHA256,
                    SignatureScheme::RSA_PSS_SHA384,
                    SignatureScheme::RSA_PSS_SHA512,
                    SignatureScheme::ED25519,
                ]
            }
        }

        let mut cfg = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerify))
            .with_no_client_auth();
        cfg.alpn_protocols = alpn.iter().map(|p| p.as_bytes().to_vec()).collect();
        Arc::new(cfg)
    }

    /// Spawn a TLS-enabled SWS server in a background thread.
    ///
    /// Pre-binds a TCP listener on port 0 and passes it to the server via
    /// [`Server::with_pre_bound_listener`], eliminating TOCTOU port races.
    pub fn spawn_server(
        extra_args: &[&str],
    ) -> (
        u16,
        tokio::sync::watch::Sender<()>,
        std::thread::JoinHandle<static_web_server::Result>,
    ) {
        const CERT: &str = "tests/tls/local.dev_cert.pkcs8.pem";
        const KEY: &str = "tests/tls/local.dev_key.pkcs8.pem";

        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());

        // Pre-bind a listener so the port is reserved until the server takes
        // ownership. This avoids the race between `free_port()` and the
        // server's own bind call.
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("bind to port 0 for free port");
        let port = listener.local_addr().unwrap().port();

        let mut args: Vec<String> = [
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--tls",
            "--tls-cert",
            CERT,
            "--tls-key",
            KEY,
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        for a in extra_args {
            args.push(a.to_string());
        }

        let handle = std::thread::spawn(move || {
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let settings =
                static_web_server::Settings::get_unparsed(false, &refs).expect("settings parse");
            static_web_server::Server::new(settings)
                .expect("server build")
                .with_pre_bound_listener(listener)
                .run_server_on_rt(Some(cancel_rx), || {}, false)
        });

        (port, cancel_tx, handle)
    }

    /// Spawn a plain HTTP/1 (no TLS) server.
    ///
    /// Pre-binds a TCP listener on port 0 and passes it to the server via
    /// [`Server::with_pre_bound_listener`], eliminating TOCTOU port races.
    pub fn spawn_plain_server(
        extra_args: &[&str],
    ) -> (
        u16,
        tokio::sync::watch::Sender<()>,
        std::thread::JoinHandle<static_web_server::Result>,
    ) {
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());

        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("bind to port 0 for free port");
        let port = listener.local_addr().unwrap().port();

        let mut args: Vec<String> = [
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        for a in extra_args {
            args.push(a.to_string());
        }

        let handle = std::thread::spawn(move || {
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let settings =
                static_web_server::Settings::get_unparsed(false, &refs).expect("settings parse");
            static_web_server::Server::new(settings)
                .expect("server build")
                .with_pre_bound_listener(listener)
                .run_server_on_rt(Some(cancel_rx), || {}, false)
        });

        (port, cancel_tx, handle)
    }

    /// Send cancel and join the server thread with a timeout.
    pub async fn shutdown_server(
        cancel_tx: tokio::sync::watch::Sender<()>,
        handle: std::thread::JoinHandle<static_web_server::Result>,
    ) {
        let _ = cancel_tx.send(());
        // Join in a blocking task to avoid starving the async runtime.
        let join_result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::task::spawn_blocking(move || handle.join().expect("server thread panicked")),
        )
        .await;

        match join_result {
            Ok(Ok(server_result)) => {
                server_result.ok();
            }
            Ok(Err(e)) => panic!("spawn_blocking panicked: {e:?}"),
            Err(_) => {
                // Server didn't shut down within 5s — not ideal but don't block the test.
                eprintln!("warning: server did not shut down within 5s after cancel");
            }
        }
    }
}

#[cfg(feature = "tls")]
#[cfg(test)]
mod live_http1_plain_tests {
    use http::StatusCode;
    use http_body_util::Empty;
    use hyper::Request;
    use hyper::body::Incoming;
    use hyper_util::rt::TokioIo;

    use super::test_helpers::*;

    async fn http_get(port: u16, path: &str) -> StatusCode {
        http_get_response(port, path).await.status()
    }

    async fn http_get_response(port: u16, path: &str) -> hyper::Response<Incoming> {
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");

        let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .expect("HTTP/1 handshake");
        tokio::spawn(conn);

        let req = Request::builder()
            .uri(path)
            .header("host", format!("localhost:{port}"))
            .body(Empty::<bytes::Bytes>::new())
            .unwrap();

        sender.send_request(req).await.expect("send request")
    }

    #[tokio::test]
    async fn http1_plain_omits_hsts() {
        let (port, cancel_tx, handle) = spawn_plain_server(&["--security-headers=true"]);
        wait_for_server(port).await;

        let resp = tokio::time::timeout(TIMEOUT, http_get_response(port, "/index.htm"))
            .await
            .expect("request timed out");
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().contains_key("x-frame-options"));
        assert!(
            !resp.headers().contains_key("strict-transport-security"),
            "HSTS must not be sent over plain HTTP"
        );

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn http1_plain_serves_files() {
        let (port, cancel_tx, handle) = spawn_plain_server(&[]);
        wait_for_server(port).await;

        let status = tokio::time::timeout(TIMEOUT, http_get(port, "/index.htm"))
            .await
            .expect("request timed out");
        assert_eq!(status, StatusCode::OK);

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn http1_plain_returns_404() {
        let (port, cancel_tx, handle) = spawn_plain_server(&[]);
        wait_for_server(port).await;

        let status = tokio::time::timeout(TIMEOUT, http_get(port, "/no-such-file.html"))
            .await
            .expect("request timed out");
        assert_eq!(status, StatusCode::NOT_FOUND);

        shutdown_server(cancel_tx, handle).await;
    }
}

#[cfg(feature = "tls")]
#[cfg(test)]
mod live_http1_tls_tests {
    use http::StatusCode;
    use http_body_util::Empty;
    use hyper::Request;
    use hyper::body::Incoming;
    use hyper_util::rt::TokioIo;
    use rustls_pki_types::ServerName;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_rustls::TlsConnector;

    use super::test_helpers::*;

    async fn https_get_http1(port: u16, path: &str) -> StatusCode {
        https_get_http1_response(port, path).await.status()
    }

    async fn https_get_http1_response(port: u16, path: &str) -> hyper::Response<Incoming> {
        let cfg = tls_client_config(&["http/1.1"]);
        let connector = TlsConnector::from(cfg);
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        let domain = ServerName::try_from("localhost").unwrap().to_owned();
        let tls_stream = connector
            .connect(domain, stream)
            .await
            .expect("TLS handshake");

        let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(tls_stream))
            .await
            .expect("HTTP/1 handshake");
        tokio::spawn(conn);

        let req = Request::builder()
            .uri(path)
            .header("host", format!("localhost:{port}"))
            .body(Empty::<bytes::Bytes>::new())
            .unwrap();

        sender.send_request(req).await.expect("send request")
    }

    #[tokio::test]
    async fn sends_hsts_without_preload() {
        let (port, cancel_tx, handle) = spawn_server(&[]);
        wait_for_server(port).await;

        let resp = tokio::time::timeout(TIMEOUT, https_get_http1_response(port, "/index.htm"))
            .await
            .expect("request timed out");
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("strict-transport-security")
                .and_then(|v| v.to_str().ok()),
            Some("max-age=63072000; includeSubDomains")
        );

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn security_headers_false_disables_them() {
        let (port, cancel_tx, handle) = spawn_server(&["--security-headers=false"]);
        wait_for_server(port).await;

        let resp = tokio::time::timeout(TIMEOUT, https_get_http1_response(port, "/index.htm"))
            .await
            .expect("request timed out");
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(!resp.headers().contains_key("strict-transport-security"));
        assert!(!resp.headers().contains_key("x-frame-options"));

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn serves_files() {
        let (port, cancel_tx, handle) = spawn_server(&[]);
        wait_for_server(port).await;

        let status = tokio::time::timeout(TIMEOUT, https_get_http1(port, "/index.htm"))
            .await
            .expect("request timed out");
        assert_eq!(status, StatusCode::OK);

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn returns_404_for_missing_file() {
        let (port, cancel_tx, handle) = spawn_server(&[]);
        wait_for_server(port).await;

        let status = tokio::time::timeout(TIMEOUT, https_get_http1(port, "/no-such-file.html"))
            .await
            .expect("request timed out");
        assert_eq!(status, StatusCode::NOT_FOUND);

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn tls_handshake_completes() {
        let (port, cancel_tx, handle) = spawn_server(&[]);
        wait_for_server(port).await;

        let cfg = tls_client_config(&["http/1.1"]);
        let connector = TlsConnector::from(cfg);
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        let domain = ServerName::try_from("localhost").unwrap().to_owned();
        let result = tokio::time::timeout(TIMEOUT, connector.connect(domain, stream)).await;
        assert!(
            result.is_ok() && result.unwrap().is_ok(),
            "TLS handshake should complete"
        );

        shutdown_server(cancel_tx, handle).await;
    }

    #[test]
    fn mismatched_cert_key_fails_to_start() {
        let port = free_port();
        let (_, cancel_rx) = tokio::sync::watch::channel(());
        let port_str = port.to_string();
        let result = std::thread::spawn(move || {
            let settings = static_web_server::Settings::get_unparsed(
                false,
                &[
                    "static-web-server",
                    "--root",
                    "tests/fixtures/public",
                    "--host",
                    "127.0.0.1",
                    "--port",
                    &port_str,
                    "--tls",
                    "--tls-cert",
                    "tests/tls/local.dev_cert.rsa_pkcs1.pem",
                    "--tls-key",
                    "tests/tls/local.dev_key.sec1_ec.pem",
                ],
            )
            .expect("settings must parse");
            static_web_server::Server::new(settings)
                .expect("server must build")
                .run_server_on_rt(Some(cancel_rx), || {}, false)
        })
        .join()
        .expect("thread should not panic");

        assert!(
            result.is_err(),
            "server with mismatched cert/key should fail to start"
        );
    }

    #[tokio::test]
    async fn idle_timeout_closes_connection_without_handshake() {
        let (port, cancel_tx, handle) = spawn_server(&["--idle-timeout", "1"]);
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");

        // The client never starts the TLS handshake, so the server must close the connection.
        let mut buf = [0u8; 1];
        let read = tokio::time::timeout(TIMEOUT, stream.read(&mut buf))
            .await
            .expect("connection without a handshake was not closed");
        assert!(
            matches!(read, Ok(0) | Err(_)),
            "server sent data instead of closing the connection"
        );

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn max_connections_is_shared_with_https_redirect() {
        let redirect_port = free_port().to_string();
        let (port, cancel_tx, handle) = spawn_server(&[
            "--max-connections",
            "1",
            "--https-redirect",
            "--https-redirect-from-port",
            &redirect_port,
        ]);
        let redirect_port: u16 = redirect_port.parse().unwrap();
        wait_for_server(port).await;
        wait_for_server(redirect_port).await;

        // The HTTPS connection takes the only slot, so the redirect server doesn't answer.
        let https = tls_connect(port, &["http/1.1"]).await;
        let mut http = tokio::net::TcpStream::connect(("127.0.0.1", redirect_port))
            .await
            .expect("TCP connect");
        http.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .expect("write request");
        let mut buf = [0u8; 1024];
        assert!(
            tokio::time::timeout(Duration::from_millis(500), http.read(&mut buf))
                .await
                .is_err(),
            "redirect server answered past the shared limit"
        );

        drop(https);
        let n = tokio::time::timeout(TIMEOUT, http.read(&mut buf))
            .await
            .expect("redirect server did not answer after the HTTPS connection closed")
            .expect("read response");
        let response = String::from_utf8_lossy(&buf[..n]);
        assert!(response.starts_with("HTTP/1.1 301"), "{response}");

        drop(http);
        shutdown_server(cancel_tx, handle).await;
    }
}

#[cfg(all(feature = "tls", feature = "http2"))]
#[cfg(test)]
mod live_http2_tls_tests {
    use http::StatusCode;
    use http_body_util::Empty;
    use hyper::Request;
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use rustls_pki_types::ServerName;
    use tokio::io::AsyncReadExt;
    use tokio_rustls::TlsConnector;

    use super::test_helpers::*;

    async fn https_get_http2(port: u16, path: &str) -> StatusCode {
        let req = Request::builder()
            .uri(format!("https://localhost:{port}{path}"))
            .body(Empty::<bytes::Bytes>::new())
            .unwrap();
        https_send_http2(port, req).await
    }

    async fn https_send_http2(port: u16, req: Request<Empty<bytes::Bytes>>) -> StatusCode {
        let cfg = tls_client_config(&["h2"]);
        let connector = TlsConnector::from(cfg);
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        let domain = ServerName::try_from("localhost").unwrap().to_owned();
        let tls_stream = connector
            .connect(domain, stream)
            .await
            .expect("TLS handshake");

        let (mut sender, conn) =
            hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(tls_stream))
                .await
                .expect("HTTP/2 handshake");
        tokio::spawn(conn);

        sender
            .send_request(req)
            .await
            .expect("send request")
            .status()
    }

    #[tokio::test]
    async fn serves_files() {
        let (port, cancel_tx, handle) = spawn_server(&["--http2"]);
        wait_for_server(port).await;

        let status = tokio::time::timeout(TIMEOUT, https_get_http2(port, "/index.htm"))
            .await
            .expect("request timed out");
        assert_eq!(status, StatusCode::OK);

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn returns_404_for_missing_file() {
        let (port, cancel_tx, handle) = spawn_server(&["--http2"]);
        wait_for_server(port).await;

        let status = tokio::time::timeout(TIMEOUT, https_get_http2(port, "/no-such-file.html"))
            .await
            .expect("request timed out");
        assert_eq!(status, StatusCode::NOT_FOUND);

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn max_header_size_limits_header_list() {
        let (port, cancel_tx, handle) = spawn_server(&["--http2", "--max-header-size", "32768"]);
        wait_for_server(port).await;

        // 20 KB is above the 16 KiB HTTP/2 default but within the configured limit.
        for (pad, expected) in [
            (20_000, StatusCode::OK),
            (40_000, StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE),
        ] {
            let req = Request::builder()
                .uri(format!("https://localhost:{port}/index.htm"))
                .header("x-pad", "a".repeat(pad))
                .body(Empty::<bytes::Bytes>::new())
                .unwrap();
            let status = tokio::time::timeout(TIMEOUT, https_send_http2(port, req))
                .await
                .expect("request timed out");
            assert_eq!(status, expected, "x-pad header of {pad} bytes");
        }

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn tls_handshake_with_h2_alpn() {
        let (port, cancel_tx, handle) = spawn_server(&["--http2"]);
        wait_for_server(port).await;

        let cfg = tls_client_config(&["h2"]);
        let connector = TlsConnector::from(cfg);
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        let domain = ServerName::try_from("localhost").unwrap().to_owned();
        let result = tokio::time::timeout(TIMEOUT, connector.connect(domain, stream)).await;
        assert!(
            result.is_ok() && result.unwrap().is_ok(),
            "TLS handshake with h2 ALPN should succeed"
        );

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn idle_timeout_closes_connection_without_preface() {
        let (port, cancel_tx, handle) = spawn_server(&["--http2", "--idle-timeout", "1"]);
        wait_for_server(port).await;

        let connector = TlsConnector::from(tls_client_config(&["h2"]));
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        let domain = ServerName::try_from("localhost").unwrap().to_owned();
        let mut tls_stream = connector
            .connect(domain, stream)
            .await
            .expect("TLS handshake");

        // The client never sends the HTTP/2 preface, so the server must close the connection.
        let mut buf = [0u8; 1024];
        let read = tokio::time::timeout(TIMEOUT, async {
            loop {
                match tls_stream.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        })
        .await;
        assert!(read.is_ok(), "connection without a preface was not closed");

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn idle_timeout_closes_idle_connection() {
        let (port, cancel_tx, handle) = spawn_server(&["--http2", "--idle-timeout", "1"]);
        wait_for_server(port).await;

        let connector = TlsConnector::from(tls_client_config(&["h2"]));
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        let domain = ServerName::try_from("localhost").unwrap().to_owned();
        let tls_stream = connector
            .connect(domain, stream)
            .await
            .expect("TLS handshake");
        let (mut sender, conn) =
            hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(tls_stream))
                .await
                .expect("HTTP/2 handshake");
        let conn = tokio::spawn(conn);

        let req = Request::builder()
            .uri(format!("https://localhost:{port}/index.htm"))
            .body(Empty::<bytes::Bytes>::new())
            .unwrap();
        let status = sender
            .send_request(req)
            .await
            .expect("send request")
            .status();
        assert_eq!(status, StatusCode::OK);

        // The connection stays idle after the response, so the server must close it.
        let closed = tokio::time::timeout(TIMEOUT, conn).await;
        assert!(closed.is_ok(), "idle connection was not closed");

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn max_connections_holds_new_connections_until_one_closes() {
        let (port, cancel_tx, handle) = spawn_server(&["--http2", "--max-connections", "1"]);
        wait_for_server(port).await;

        let first = tls_connect(port, &["h2"]).await;

        // The first connection takes the only slot, so the second handshake doesn't complete.
        let second = tls_connect(port, &["h2"]);
        tokio::pin!(second);
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(500), &mut second)
                .await
                .is_err(),
            "second connection accepted past the limit"
        );

        drop(first);
        let second = tokio::time::timeout(TIMEOUT, second)
            .await
            .expect("second connection not accepted after the first one closed");

        drop(second);
        shutdown_server(cancel_tx, handle).await;
    }
}
