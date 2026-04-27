#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

/// Grab a free local port by briefly binding to 0 and reading back the
/// OS-assigned port. The listener is immediately dropped so the port is
/// available when the server binds to it.
#[cfg(feature = "tls")]
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("failed to find a free port")
        .local_addr()
        .unwrap()
        .port()
}

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

// Live server tests

/// Live-server tests that spin up a real TLS listener, assert that TCP
/// connections are accepted and then shut the server down via a cancel channel.
#[cfg(feature = "tls")]
#[cfg(test)]
mod live_server_tests {
    use super::free_port;
    use static_web_server::{Server, Settings};
    use std::time::Duration;

    const CERT: &str = "tests/tls/local.dev_cert.pkcs8.pem";
    const KEY: &str = "tests/tls/local.dev_key.pkcs8.pem";

    /// Start an HTTP/1+TLS server on a free port, verify that a raw TCP
    /// connection is accepted (proving the listener is up), then cancel the
    /// server and confirm it exits cleanly.
    #[test]
    fn http1_tls_server_starts_and_accepts_connections() {
        let port = free_port();

        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());

        let port_str = port.to_string();
        let server_thread = std::thread::spawn(move || {
            let settings = Settings::get_unparsed(
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
                    CERT,
                    "--tls-key",
                    KEY,
                ],
            )
            .expect("settings must parse");

            Server::new(settings)
                .expect("server must build")
                .run_server_on_rt(Some(cancel_rx), || {}, false)
        });

        // Allow the server time to bind and start its accept loop.
        std::thread::sleep(Duration::from_millis(300));

        // A successful TCP connect proves the server is listening.
        std::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .expect("TLS server should accept TCP connections");

        // Signal a graceful shutdown.
        cancel_tx.send(()).expect("cancel signal must be sent");

        let result = server_thread
            .join()
            .expect("server thread should not panic");
        assert!(
            result.is_ok(),
            "server should shut down cleanly, got: {:?}",
            result
        );
    }

    /// Starting a TLS server with a cert/key pair of a different algorithm
    /// (e.g. RSA cert with EC key) must fail before accepting any connections.
    #[test]
    fn http1_tls_server_mismatched_cert_key_fails_to_start() {
        let port = free_port();

        let (_, cancel_rx) = tokio::sync::watch::channel(());

        let port_str = port.to_string();
        let result = std::thread::spawn(move || {
            let settings = Settings::get_unparsed(
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

            Server::new(settings)
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

/// Live HTTP/2+TLS server tests.
#[cfg(all(feature = "tls", feature = "http2"))]
#[cfg(test)]
mod live_http2_server_tests {
    use super::free_port;
    use static_web_server::{Server, Settings};
    use std::time::Duration;

    const CERT: &str = "tests/tls/local.dev_cert.pkcs8.pem";
    const KEY: &str = "tests/tls/local.dev_key.pkcs8.pem";

    /// Start an HTTP/2+TLS server on a free port, verify that a raw TCP
    /// connection is accepted (proving the listener is up), then cancel the
    /// server and confirm it exits cleanly.
    #[test]
    fn http2_tls_server_starts_and_accepts_connections() {
        let port = free_port();

        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());

        let port_str = port.to_string();
        let server_thread = std::thread::spawn(move || {
            let settings = Settings::get_unparsed(
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
                    CERT,
                    "--tls-key",
                    KEY,
                    "--http2",
                ],
            )
            .expect("settings must parse");

            Server::new(settings)
                .expect("server must build")
                .run_server_on_rt(Some(cancel_rx), || {}, false)
        });

        // Allow the server time to bind and start its accept loop.
        std::thread::sleep(Duration::from_millis(300));

        // A successful TCP connect proves the server is listening.
        std::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .expect("HTTP/2+TLS server should accept TCP connections");

        // Signal a graceful shutdown.
        cancel_tx.send(()).expect("cancel signal must be sent");

        let result = server_thread
            .join()
            .expect("server thread should not panic");
        assert!(
            result.is_ok(),
            "HTTP/2+TLS server should shut down cleanly, got: {:?}",
            result
        );
    }
}
