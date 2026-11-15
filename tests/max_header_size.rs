#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
    use tokio::net::TcpStream;

    const STATUS_OK: &str = "HTTP/1.1 200 OK";
    const STATUS_TOO_LARGE: &str = "HTTP/1.1 431 Request Header Fields Too Large";

    /// The server's HTTP/1 buffer size (`HTTP1_MAX_BUF_SIZE` in `src/server.rs`).
    const HTTP1_MAX_BUF_SIZE: usize = 64 * 1024;

    /// Complete heads far enough above the buffer size to be rejected, however they arrive.
    ///
    /// A complete head that arrives in few segments can exceed the buffer size by up to
    /// about 4 times, so these stay above that.
    const OVERSIZED_HEADS: [usize; 2] = [300_000, 500_000];

    /// Run one test server at a time.
    ///
    /// The server can't take a pre-bound listener, so a free port is found by binding and
    /// dropping a listener first. Running the tests one at a time keeps two of them from
    /// getting the same port and probing each other's server.
    static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    /// A server running on its own thread.
    struct TestServer {
        port: u16,
        cancel_tx: tokio::sync::watch::Sender<()>,
        handle: std::thread::JoinHandle<static_web_server::Result>,
        _serial: tokio::sync::MutexGuard<'static, ()>,
    }

    impl TestServer {
        /// Stop the server and wait for its thread to finish.
        ///
        /// On Windows the cancel receiver is only used when running as a Windows service,
        /// so the server isn't waited for and keeps running until the test process exits.
        async fn shutdown(self) {
            let _ = self.cancel_tx.send(());

            #[cfg(windows)]
            drop(self.handle);

            #[cfg(not(windows))]
            {
                let handle = self.handle;
                tokio::time::timeout(
                    Duration::from_secs(5),
                    tokio::task::spawn_blocking(move || {
                        handle.join().expect("server thread panicked")
                    }),
                )
                .await
                .expect("server did not shut down")
                .expect("join task failed")
                .expect("server returned an error");
            }
        }

        /// Ask the server to stop without waiting for it.
        ///
        /// The HTTPS redirect server shares the cancel receiver with the HTTP/2 server and doesn't
        /// see the change once the HTTP/2 server has consumed it, so the thread is left running.
        #[cfg(feature = "http2")]
        fn cancel(self) {
            let _ = self.cancel_tx.send(());
        }
    }

    /// Find a free local port.
    fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind to a free port")
            .local_addr()
            .expect("listener address")
            .port()
    }

    /// Spawn a server on a free port with the given extra arguments and wait until it accepts connections.
    async fn spawn_server(extra_args: &[&str]) -> TestServer {
        let serial = SERIAL.lock().await;
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());
        let port = free_port();

        let mut args: Vec<String> = vec![
            "static-web-server".into(),
            "--root".into(),
            "tests/fixtures/public".into(),
            "--host".into(),
            "127.0.0.1".into(),
            "--port".into(),
            port.to_string(),
        ];
        args.extend(extra_args.iter().map(|s| s.to_string()));

        let handle = std::thread::spawn(move || {
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let settings =
                static_web_server::Settings::get_unparsed(false, &refs).expect("settings parse");
            static_web_server::Server::new(settings)
                .expect("server build")
                .run_server_on_rt(Some(cancel_rx), || {}, false)
        });

        wait_for_port(port).await;
        // A server that failed to bind its port has already returned.
        assert!(
            !handle.is_finished(),
            "server on port {port} exited during startup"
        );

        TestServer {
            port,
            cancel_tx,
            handle,
            _serial: serial,
        }
    }

    /// Wait until a TCP connection to the port succeeds.
    async fn wait_for_port(port: u16) {
        for _ in 0..100 {
            if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("server on port {port} did not start");
    }

    /// Build a request head of exactly `len` bytes, padded by a single `X-Pad` header.
    ///
    /// With `complete` set the head ends with an empty line, otherwise it stops mid-header.
    fn request_head(len: usize, complete: bool) -> Vec<u8> {
        let start = b"GET /index.htm HTTP/1.1\r\nHost: localhost\r\nX-Pad: ";
        let end: &[u8] = if complete {
            b"\r\nConnection: close\r\n\r\n"
        } else {
            b""
        };
        let mut head = start.to_vec();
        head.resize(len - end.len(), b'a');
        head.extend_from_slice(end);
        assert_eq!(head.len(), len);
        head
    }

    /// Send `head` over `stream` and return the response status line, or an empty string
    /// if the server closes or resets the connection without an answer.
    ///
    /// The head is written by a separate task, so a server that stops reading can't block it.
    async fn exchange<S>(stream: S, head: Vec<u8>) -> String
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        let (mut reader, mut writer) = tokio::io::split(stream);
        let write_task = tokio::spawn(async move {
            let _ = writer.write_all(&head).await;
            let _ = writer.flush().await;
            writer
        });

        let mut received = Vec::new();
        let _ = tokio::time::timeout(Duration::from_secs(5), async {
            let mut buf = [0u8; 4096];
            while let Ok(n @ 1..) = reader.read(&mut buf).await {
                received.extend_from_slice(&buf[..n]);
            }
        })
        .await;
        write_task.abort();

        let response = String::from_utf8_lossy(&received);
        response.lines().next().unwrap_or_default().to_owned()
    }

    /// Send `head` over a plain TCP connection and return the response status line.
    async fn send(port: u16, head: Vec<u8>) -> String {
        let stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        exchange(stream, head).await
    }

    #[tokio::test]
    async fn max_header_size_accepts_head_up_to_limit() {
        let server = spawn_server(&[]).await;

        assert_eq!(
            send(server.port, request_head(8 * 1024, true)).await,
            STATUS_OK
        );
        assert_eq!(
            send(server.port, request_head(HTTP1_MAX_BUF_SIZE, true)).await,
            STATUS_OK
        );

        server.shutdown().await;
    }

    #[tokio::test]
    async fn max_header_size_rejects_larger_head() {
        let server = spawn_server(&[]).await;

        // An incomplete head that reaches the limit can't fit anymore. The server reads all of it
        // before replying, so the 431 isn't lost to a connection reset.
        let status = send(server.port, request_head(HTTP1_MAX_BUF_SIZE, false)).await;
        assert_eq!(status, STATUS_TOO_LARGE);

        // A much larger head is rejected before the server reads all of it, so the answer can
        // be lost to a connection reset, but it must never be served.
        for len in OVERSIZED_HEADS {
            let status = send(server.port, request_head(len, true)).await;
            assert!(
                status.is_empty() || status == STATUS_TOO_LARGE,
                "head of {len} bytes got {status:?}"
            );
        }

        server.shutdown().await;
    }

    #[cfg(feature = "http2")]
    mod tls {
        use super::*;
        use std::sync::Arc;
        use tokio_rustls::TlsConnector;
        use tokio_rustls::rustls::client::danger::{
            HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
        };
        use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
        use tokio_rustls::rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};

        const TLS_ARGS: [&str; 5] = [
            "--http2",
            "--http2-tls-cert",
            "tests/tls/local.dev_cert.pkcs8.pem",
            "--http2-tls-key",
            "tests/tls/local.dev_key.pkcs8.pem",
        ];

        /// Accept the self-signed test certificate.
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
            ) -> Result<ServerCertVerified, Error> {
                Ok(ServerCertVerified::assertion())
            }

            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
                vec![
                    SignatureScheme::ECDSA_NISTP256_SHA256,
                    SignatureScheme::ECDSA_NISTP384_SHA384,
                    SignatureScheme::ED25519,
                    SignatureScheme::RSA_PSS_SHA256,
                    SignatureScheme::RSA_PSS_SHA384,
                    SignatureScheme::RSA_PSS_SHA512,
                    SignatureScheme::RSA_PKCS1_SHA256,
                    SignatureScheme::RSA_PKCS1_SHA384,
                    SignatureScheme::RSA_PKCS1_SHA512,
                ]
            }
        }

        /// Send `head` over a TLS connection that negotiates HTTP/1.1 and return the response status line.
        async fn send_tls(port: u16, head: Vec<u8>) -> String {
            let mut config = ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoVerify))
                .with_no_client_auth();
            config.alpn_protocols = vec![b"http/1.1".to_vec()];

            let stream = TcpStream::connect(("127.0.0.1", port))
                .await
                .expect("TCP connect");
            let stream = TlsConnector::from(Arc::new(config))
                .connect(ServerName::try_from("localhost").unwrap(), stream)
                .await
                .expect("TLS handshake");
            assert_eq!(stream.get_ref().1.alpn_protocol(), Some(&b"http/1.1"[..]));
            exchange(stream, head).await
        }

        #[tokio::test]
        async fn max_header_size_tls_http1() {
            let server = spawn_server(&TLS_ARGS).await;

            let status = send_tls(server.port, request_head(8 * 1024, true)).await;
            assert_eq!(status, STATUS_OK);
            let status = send_tls(server.port, request_head(HTTP1_MAX_BUF_SIZE, true)).await;
            assert_eq!(status, STATUS_OK);
            let status = send_tls(server.port, request_head(HTTP1_MAX_BUF_SIZE, false)).await;
            assert_eq!(status, STATUS_TOO_LARGE);
            for len in OVERSIZED_HEADS {
                let status = send_tls(server.port, request_head(len, true)).await;
                assert!(
                    status.is_empty() || status == STATUS_TOO_LARGE,
                    "head of {len} bytes got {status:?}"
                );
            }

            server.shutdown().await;
        }

        #[tokio::test]
        async fn max_header_size_https_redirect_server() {
            let redirect_port = free_port();
            let redirect_port_arg = redirect_port.to_string();
            let mut args = TLS_ARGS.to_vec();
            args.extend([
                "--https-redirect",
                "--https-redirect-from-port",
                &redirect_port_arg,
            ]);
            let server = spawn_server(&args).await;
            wait_for_port(redirect_port).await;

            let status = send(redirect_port, request_head(HTTP1_MAX_BUF_SIZE, true)).await;
            assert_eq!(status, "HTTP/1.1 301 Moved Permanently");
            let status = send(redirect_port, request_head(HTTP1_MAX_BUF_SIZE, false)).await;
            assert_eq!(status, STATUS_TOO_LARGE);
            for len in OVERSIZED_HEADS {
                let status = send(redirect_port, request_head(len, true)).await;
                assert!(
                    status.is_empty() || status == STATUS_TOO_LARGE,
                    "head of {len} bytes got {status:?}"
                );
            }

            server.cancel();
        }
    }
}
