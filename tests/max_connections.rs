#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    const REQUEST: &[u8] = b"GET /index.htm HTTP/1.1\r\nHost: localhost\r\n\r\n";

    /// Spawn a plain HTTP/1 server on a pre-bound port with the given connection limit.
    fn spawn_server(
        max_connections: &str,
    ) -> (
        u16,
        tokio::sync::watch::Sender<()>,
        std::thread::JoinHandle<static_web_server::Result>,
    ) {
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind to a free port");
        let port = listener.local_addr().expect("listener address").port();

        let args = [
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--max-connections",
            max_connections,
        ]
        .map(String::from);

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

    /// Stop the server and wait for its thread to finish.
    async fn shutdown_server(
        cancel_tx: tokio::sync::watch::Sender<()>,
        handle: std::thread::JoinHandle<static_web_server::Result>,
    ) {
        let _ = cancel_tx.send(());
        tokio::time::timeout(
            Duration::from_secs(5),
            tokio::task::spawn_blocking(move || handle.join().expect("server thread panicked")),
        )
        .await
        .expect("server did not shut down")
        .expect("join task failed")
        .expect("server returned an error");
    }

    /// Read the status line of a response, or `None` if none arrives within `wait`.
    async fn status_line(stream: &mut TcpStream, wait: Duration) -> Option<String> {
        let mut buf = [0u8; 1024];
        let read = tokio::time::timeout(wait, stream.read(&mut buf))
            .await
            .ok()?;
        let n = read.expect("read response");
        let response = String::from_utf8_lossy(&buf[..n]);
        response.lines().next().map(String::from)
    }

    /// Connect and send a keep-alive request.
    async fn send_request(port: u16) -> TcpStream {
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        stream.write_all(REQUEST).await.expect("write request");
        stream
    }

    #[tokio::test]
    async fn max_connections_holds_new_connections_until_one_closes() {
        let (port, cancel_tx, handle) = spawn_server("1");

        let mut first = send_request(port).await;
        assert_eq!(
            status_line(&mut first, Duration::from_secs(5)).await,
            Some("HTTP/1.1 200 OK".into())
        );

        // The first connection stays open (keep-alive), so the second one isn't accepted yet.
        let mut second = send_request(port).await;
        assert_eq!(
            status_line(&mut second, Duration::from_millis(500)).await,
            None,
            "second connection served past the limit"
        );

        drop(first);
        assert_eq!(
            status_line(&mut second, Duration::from_secs(5)).await,
            Some("HTTP/1.1 200 OK".into()),
            "second connection not served after the first one closed"
        );

        drop(second);
        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn max_connections_zero_disables_limit() {
        let (port, cancel_tx, handle) = spawn_server("0");

        let mut first = send_request(port).await;
        let mut second = send_request(port).await;
        for stream in [&mut first, &mut second] {
            assert_eq!(
                status_line(stream, Duration::from_secs(5)).await,
                Some("HTTP/1.1 200 OK".into())
            );
        }

        drop((first, second));
        shutdown_server(cancel_tx, handle).await;
    }

    #[test]
    fn max_connections_rejects_out_of_range_value() {
        // Above the limit on 32 and 64-bit targets, and still a valid `usize`.
        let max = usize::MAX.to_string();
        let args = ["static-web-server", "--max-connections", &max];
        let settings =
            static_web_server::Settings::get_unparsed(false, &args).expect("settings parse");
        let err = static_web_server::Server::new(settings)
            .expect("server build")
            .run_server_on_rt(None, || {}, false)
            .expect_err("server started with an out-of-range limit");
        assert!(
            err.to_string().contains("max connections must be at most"),
            "{err}"
        );
    }
}
