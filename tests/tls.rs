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

    /// Enabling TLS must automatically set `security_headers = true` even if
    /// the user did not supply `--security-headers`.
    #[cfg(feature = "tls")]
    #[test]
    fn tls_auto_enables_security_headers() {
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

        assert!(
            settings.general.security_headers,
            "--tls should automatically enable security_headers"
        );
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
    use hyper_util::rt::TokioIo;

    use super::test_helpers::*;

    async fn http_get(port: u16, path: &str) -> StatusCode {
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

        sender
            .send_request(req)
            .await
            .expect("send request")
            .status()
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
    use hyper_util::rt::TokioIo;
    use rustls_pki_types::ServerName;
    use tokio_rustls::TlsConnector;

    use super::test_helpers::*;

    async fn https_get_http1(port: u16, path: &str) -> StatusCode {
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

        sender
            .send_request(req)
            .await
            .expect("send request")
            .status()
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
}

#[cfg(all(feature = "tls", feature = "http2"))]
#[cfg(test)]
mod live_http2_tls_tests {
    use http::StatusCode;
    use http_body_util::Empty;
    use hyper::Request;
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use rustls_pki_types::ServerName;
    use tokio_rustls::TlsConnector;

    use super::test_helpers::*;

    async fn https_get_http2(port: u16, path: &str) -> StatusCode {
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

        let req = Request::builder()
            .uri(format!("https://localhost:{port}{path}"))
            .body(Empty::<bytes::Bytes>::new())
            .unwrap();

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
}
