#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpSocket, TcpStream};

    const FIXTURES_ROOT: &str = "tests/fixtures/public";

    /// Spawn a plain HTTP/1 server on a pre-bound port with the given root and idle timeout.
    fn spawn_server(
        root: &str,
        idle_timeout: &str,
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
            root,
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--idle-timeout",
            idle_timeout,
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
        // Join in a blocking task to avoid starving the async runtime.
        let join_result = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::task::spawn_blocking(move || handle.join().expect("server thread panicked")),
        )
        .await;

        match join_result {
            Ok(Ok(server_result)) => {
                server_result.expect("server returned an error");
            }
            Ok(Err(e)) => panic!("spawn_blocking panicked: {e:?}"),
            Err(_) => {
                // Server didn't shut down within 5s — not ideal but don't block the test.
                eprintln!("warning: server did not shut down within 5s after cancel");
            }
        }
    }

    /// Read from the stream until the server closes it, or until `wait` elapses.
    ///
    /// Returns the bytes read and whether the server closed the connection.
    async fn read_until_closed(stream: &mut TcpStream, wait: Duration) -> (Vec<u8>, bool) {
        let mut received = Vec::new();
        let closed = tokio::time::timeout(wait, async {
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => received.extend_from_slice(&buf[..n]),
                }
            }
        })
        .await
        .is_ok();
        (received, closed)
    }

    #[tokio::test]
    async fn idle_timeout_closes_idle_keep_alive_connection() {
        let (port, cancel_tx, handle) = spawn_server(FIXTURES_ROOT, "1");
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");

        stream
            .write_all(b"GET /index.htm HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .expect("write request");

        // The keep-alive connection stays idle after the response, so the server must close it.
        let (received, closed) = read_until_closed(&mut stream, Duration::from_secs(5)).await;
        let response = String::from_utf8_lossy(&received);
        assert!(
            response.starts_with("HTTP/1.1 200"),
            "expected 200 OK, got: {}",
            response.lines().next().unwrap_or("")
        );
        assert!(closed, "idle keep-alive connection was not closed");

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn idle_timeout_closes_connection_without_request() {
        let (port, cancel_tx, handle) = spawn_server(FIXTURES_ROOT, "1");
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");

        let (_, closed) = read_until_closed(&mut stream, Duration::from_secs(5)).await;
        assert!(closed, "connection without a request was not closed");

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn idle_timeout_closes_connection_with_slow_request_header() {
        let (port, cancel_tx, handle) = spawn_server(FIXTURES_ROOT, "1");
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");

        // Send the header a byte at a time, faster than the idle timeout,
        // so that only a limit on the whole header closes the connection.
        let start = std::time::Instant::now();
        for byte in b"GET /index.htm HTTP/1.1\r\nHost: localhost\r\n"
            .iter()
            .cycle()
        {
            if start.elapsed() > Duration::from_secs(5) || stream.write_all(&[*byte]).await.is_err()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        let (_, closed) = read_until_closed(&mut stream, Duration::from_millis(100)).await;
        assert!(
            closed,
            "connection with a slow request header was not closed"
        );

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn idle_timeout_zero_disables_timeout() {
        let (port, cancel_tx, handle) = spawn_server(FIXTURES_ROOT, "0");
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");

        let (_, closed) = read_until_closed(&mut stream, Duration::from_secs(3)).await;
        assert!(
            !closed,
            "connection was closed although the timeout is disabled"
        );

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn idle_timeout_closes_connection_that_stops_reading() {
        const FILE_SIZE: usize = 16 * 1024 * 1024;
        let root = tempfile::tempdir().expect("create root");
        std::fs::write(root.path().join("large.bin"), vec![0u8; FILE_SIZE])
            .expect("write large file");

        let (port, cancel_tx, handle) =
            spawn_server(root.path().to_str().expect("UTF-8 root"), "1");
        // A small receive buffer, so the response can't fit in the socket buffers.
        let socket = TcpSocket::new_v4().expect("create socket");
        socket
            .set_recv_buffer_size(4096)
            .expect("set receive buffer size");
        let mut stream = socket
            .connect(([127, 0, 0, 1], port).into())
            .await
            .expect("TCP connect");

        stream
            .write_all(b"GET /large.bin HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .expect("write request");

        // The client stops reading, so the server's writes make no progress and it must close the connection.
        // Without that, reading now would receive the whole file.
        tokio::time::sleep(Duration::from_secs(3)).await;
        let (received, closed) = read_until_closed(&mut stream, Duration::from_secs(5)).await;
        assert!(closed, "connection was not closed");
        assert!(
            received.len() < FILE_SIZE,
            "server sent the whole response to a client that stopped reading"
        );

        shutdown_server(cancel_tx, handle).await;
    }
}
