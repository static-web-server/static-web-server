#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    /// Spawn a plain HTTP/1 server on a pre-bound port with the given extra arguments.
    fn spawn_server(
        extra_args: &[&str],
    ) -> (
        u16,
        tokio::sync::watch::Sender<()>,
        std::thread::JoinHandle<static_web_server::Result>,
    ) {
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind to a free port");
        let port = listener.local_addr().expect("listener address").port();

        let mut args: Vec<String> = [
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
        ]
        .map(String::from)
        .to_vec();
        args.extend(extra_args.iter().map(|s| s.to_string()));

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

    /// Send `head` and return the response status line, or an empty string
    /// if the server neither answers nor closes the connection in time.
    async fn send(port: u16, head: &[u8]) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("TCP connect");
        stream.write_all(head).await.expect("write request");

        let mut received = Vec::new();
        let _ = tokio::time::timeout(Duration::from_secs(3), async {
            let mut buf = [0u8; 4096];
            while let Ok(n @ 1..) = stream.read(&mut buf).await {
                received.extend_from_slice(&buf[..n]);
            }
        })
        .await;
        let response = String::from_utf8_lossy(&received);
        response.lines().next().unwrap_or_default().to_owned()
    }

    #[tokio::test]
    async fn max_header_size_default_rejects_larger_head() {
        let (port, cancel_tx, handle) = spawn_server(&[]);

        // An incomplete head that reaches the limit can't fit anymore. The server reads all of it
        // before replying, so the 431 isn't lost to a connection reset.
        let status = send(port, &request_head(16 * 1024, false)).await;
        assert_eq!(status, "HTTP/1.1 431 Request Header Fields Too Large");

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn max_header_size_default_accepts_head_up_to_limit() {
        let (port, cancel_tx, handle) = spawn_server(&[]);

        let status = send(port, &request_head(16 * 1024, true)).await;
        assert_eq!(status, "HTTP/1.1 200 OK");

        shutdown_server(cancel_tx, handle).await;
    }

    #[tokio::test]
    async fn max_header_size_custom_value() {
        let (port, cancel_tx, handle) = spawn_server(&["--max-header-size", "8192"]);

        let status = send(port, &request_head(8192, true)).await;
        assert_eq!(status, "HTTP/1.1 200 OK");
        let status = send(port, &request_head(8192, false)).await;
        assert_eq!(status, "HTTP/1.1 431 Request Header Fields Too Large");

        shutdown_server(cancel_tx, handle).await;
    }
}
