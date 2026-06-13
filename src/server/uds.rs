// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! HTTP/1 over Unix Domain Socket accept-loop.
//!
//! This module mirrors the plain HTTP/1 accept loop in [`super::http1`] but
//! binds to a Unix Domain Socket (UDS) instead of a TCP socket. UDS is useful
//! for high-performance reverse-proxy setups (e.g. nginx → static-web-server)
//! where the front-end and back-end live on the same host: connections avoid
//! the TCP/IP stack entirely (no port allocation, no checksums, no Nagle), and
//! access can be restricted via filesystem permissions.
//!
//! The accept loop is intentionally minimal: each accepted [`tokio::net::UnixStream`]
//! is wrapped with [`hyper_util::rt::TokioIo`] and served via [`hyper::server::conn::http1`].
//! No peer socket address is propagated to the request handler since UDS peers
//! do not have a [`std::net::SocketAddr`].

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use crate::server::ShutdownCtx;
use crate::service::RouterService;
use crate::signals;
use crate::{Context, Result};

/// Run the HTTP/1 over Unix Domain Socket (UDS) accept loop until a shutdown signal
/// is received.
///
/// `socket_path` is retained so the socket file can be unlinked on graceful
/// shutdown (Unix sockets are file-system artifacts that must be cleaned up
/// explicitly, otherwise subsequent binds would fail with `EADDRINUSE`).
pub(super) async fn run<F: FnOnce()>(
    listener: UnixListener,
    socket_path: PathBuf,
    router: RouterService,
    addr_str: &str,
    threads: usize,
    ctx: ShutdownCtx,
    _cancel_fn: F,
) -> Result {
    let graceful = GracefulShutdown::new();
    let builder = http1::Builder::new();

    let signals =
        signals::create_signals().with_context(|| "failed to register termination signals")?;
    let handle = signals.handle();
    let cancel_recv = Arc::new(Mutex::new(ctx.cancel_recv));
    let shutdown = signals::wait_for_signals(signals, ctx.grace_period, cancel_recv);
    tokio::pin!(shutdown);

    tracing::info!(
        parent: tracing::info_span!("Server::start_server", ?addr_str, ?threads),
        "http1 server is listening on unix socket {}",
        addr_str
    );
    tracing::info!("press ctrl+c to shut down the server");

    loop {
        tokio::select! {
            result = listener.accept() => {
                let stream = match result {
                    Ok((stream, _addr)) => stream,
                    Err(e) => {
                        tracing::error!("failed to accept Unix socket connection: {:?}", e);
                        continue;
                    }
                };
                // UDS peers have no `SocketAddr`; pass `None` to the router so
                // downstream features (e.g. logging, real-ip headers) treat
                // this connection as having no IP-level remote address.
                let svc = router.build(None);
                let conn = builder.serve_connection(TokioIo::new(stream), svc);
                tokio::spawn(graceful.watch(conn));
            }
            _ = &mut shutdown => { break; }
        }
    }

    graceful.shutdown().await;
    handle.close();

    // Best-effort socket file cleanup. We ignore `NotFound` errors so that
    // shutting down after the path has already been unlinked (e.g. by an
    // operator or a second instance) is not treated as a failure.
    if let Err(err) = tokio::fs::remove_file(&socket_path).await
        && err.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(
            "failed to remove unix socket file {}: {:?}",
            socket_path.display(),
            err
        );
    }

    tracing::warn!("termination signal caught, shutting down the server execution");
    Ok(())
}

#[cfg(test)]
mod tests {
    //! End-to-end tests for the Unix Domain Socket accept loop.
    //!
    //! These tests bind a real `UnixListener` on a temp path, run the accept
    //! loop in a background task, drive it with raw HTTP/1.1 bytes over a
    //! `UnixStream` client, and finally trigger a graceful shutdown via the
    //! `cancel` watch channel. They exercise the full data-plane (handler,
    //! routing, response writing) over UDS without depending on Hyper's
    //! client crate.
    //!
    //! Tests are gated to `cfg(unix)` via the parent module.

    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{UnixListener, UnixStream};
    use tokio::sync::watch;

    use crate::handler::RequestHandler;
    use crate::server::{ShutdownCtx, uds};
    use crate::service::RouterService;
    use crate::testing::fixtures::{fixture_req_handler_opts, fixture_settings};

    /// Pick a per-test temp socket path. We deliberately avoid `tempfile` (not
    /// in dev-deps) and keep the path short — `sockaddr_un.sun_path` is
    /// limited to ~104 bytes on macOS, and the default `TMPDIR` on macOS CI
    /// is verbose enough to bump against that ceiling.
    fn temp_sock_path(name: &str) -> std::path::PathBuf {
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("sws-{name}-{pid}-{nanos}.sock"))
    }

    /// Build a `RouterService` backed by the same fixture handler as the rest
    /// of the integration tests so behaviour is consistent.
    fn build_router() -> RouterService {
        let opts = fixture_settings("toml/handler.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        RouterService::new(RequestHandler {
            opts: Arc::from(req_handler_opts),
        })
    }

    /// Issue a single HTTP/1.1 request over a connected UnixStream and return
    /// the raw response bytes. We rely on `Connection: close` rather than a
    /// write-side shutdown so half-close ordering can't truncate the response
    /// on slower platforms.
    async fn send_request(stream: &mut UnixStream, request: &[u8]) -> Vec<u8> {
        stream.write_all(request).await.expect("write request");
        stream.flush().await.ok();
        let mut buf = Vec::with_capacity(4096);
        stream.read_to_end(&mut buf).await.expect("read response");
        buf
    }

    #[tokio::test]
    async fn uds_serves_get_request_and_shuts_down_cleanly() {
        let path = temp_sock_path("get");
        let _ = std::fs::remove_file(&path);

        let listener = UnixListener::bind(&path).expect("bind UDS listener");
        let router = build_router();

        let (cancel_tx, cancel_rx) = watch::channel(());
        let ctx = ShutdownCtx {
            grace_period: 1,
            cancel_recv: Some(cancel_rx),
        };

        let addr_str = format!("unix:{}", path.display());
        let server_path = path.clone();
        let server = tokio::spawn(async move {
            uds::run(listener, server_path, router, &addr_str, 1, ctx, || {}).await
        });

        // Give the accept loop a moment to start before connecting.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let mut stream = UnixStream::connect(&path).await.expect("connect UDS");
        let resp = send_request(
            &mut stream,
            b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .await;

        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.starts_with("HTTP/1.1 200"),
            "expected 200 OK, got: {}",
            text.lines().next().unwrap_or("")
        );
        // The fixture serves `index.html` from `docker/public/`; assert on a
        // server-injected header to confirm the response actually came from
        // the SWS pipeline (not, e.g., a cached/empty response).
        assert!(
            text.to_lowercase().contains("server: static web server"),
            "expected Server header in response, got headers:\n{}",
            text.split("\r\n\r\n").next().unwrap_or("")
        );

        // Trigger graceful shutdown via the cancel channel.
        let _ = cancel_tx.send(());
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .expect("server did not shut down in time")
            .expect("server task panicked");
        result.expect("server returned an error");

        // The accept loop must clean up the socket file on shutdown.
        assert!(
            !path.exists(),
            "expected socket file {} to be removed on shutdown",
            path.display()
        );
    }

    #[tokio::test]
    async fn uds_returns_404_for_missing_path() {
        let path = temp_sock_path("404");
        let _ = std::fs::remove_file(&path);

        let listener = UnixListener::bind(&path).expect("bind UDS listener");
        let router = build_router();

        let (cancel_tx, cancel_rx) = watch::channel(());
        let ctx = ShutdownCtx {
            grace_period: 1,
            cancel_recv: Some(cancel_rx),
        };
        let addr_str = format!("unix:{}", path.display());
        let server_path = path.clone();
        let server = tokio::spawn(async move {
            uds::run(listener, server_path, router, &addr_str, 1, ctx, || {}).await
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let mut stream = UnixStream::connect(&path).await.expect("connect UDS");
        let resp = send_request(
            &mut stream,
            b"GET /this-does-not-exist HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .await;

        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.starts_with("HTTP/1.1 404"),
            "expected 404 Not Found, got: {}",
            text.lines().next().unwrap_or("")
        );

        let _ = cancel_tx.send(());
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), server).await;
    }
}

#[cfg(all(test, unix))]
mod uds_listener_tests {
    // Unit tests for `create_unix_listener` focused on operational safety:
    // - never clobber non-socket files,
    // - respect the `force` flag to remove stale sockets,
    // - apply requested permission bits exactly,
    // - surface long-path errors with a clear message.

    use crate::server::listener::create_unix_listener;
    use std::os::unix::fs::PermissionsExt;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("sws-uds-{name}-{pid}-{nanos}.sock"))
    }

    #[tokio::test]
    async fn binds_when_path_is_free() {
        let path = temp_path("binds");
        let _ = std::fs::remove_file(&path);
        let (listener, returned_path, addr_str) =
            create_unix_listener(&path, None, false).expect("bind UDS");
        assert_eq!(returned_path, path);
        assert!(addr_str.starts_with("unix:"));
        drop(listener);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn fails_when_path_exists_and_force_is_false() {
        let path = temp_path("nofroce");
        let _ = std::fs::remove_file(&path);
        // First bind succeeds.
        let (listener, _, _) = create_unix_listener(&path, None, false).expect("first bind");
        // Second bind without `force` must fail (EADDRINUSE bubbled up as a
        // contextualized error).
        let err = create_unix_listener(&path, None, false).expect_err("expected EADDRINUSE error");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("failed to bind unix socket"),
            "unexpected error message: {msg}"
        );
        drop(listener);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn force_removes_stale_socket_then_binds() {
        let path = temp_path("force");
        let _ = std::fs::remove_file(&path);
        // Simulate a stale socket left by a previous crashed process.
        let (listener, _, _) = create_unix_listener(&path, None, false).expect("seed bind");
        // socket file remains on disk after listener drop
        drop(listener);

        // With `force=true` the stale socket is removed and rebinding succeeds.
        let (listener, _, _) = create_unix_listener(&path, None, true).expect("rebind with force");
        drop(listener);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn force_refuses_to_remove_non_socket_files() {
        let path = temp_path("regular");
        let _ = std::fs::remove_file(&path);
        std::fs::write(&path, b"not a socket").expect("write regular file");

        let err = create_unix_listener(&path, None, true).expect_err("expected refusal");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("refusing to overwrite non-socket file"),
            "unexpected error message: {msg}"
        );
        // The regular file must remain untouched.
        assert!(path.exists());
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn applies_requested_permission_bits() {
        let path = temp_path("perms");
        let _ = std::fs::remove_file(&path);
        let (listener, _, _) =
            create_unix_listener(&path, Some(0o600), false).expect("bind with mode");
        let mode = std::fs::metadata(&path)
            .expect("stat socket")
            .permissions()
            .mode()
            & 0o7777;
        assert_eq!(mode, 0o600, "expected mode 0o600, got {mode:o}");
        drop(listener);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_paths_longer_than_sun_path_max() {
        // Build a path far exceeding the conservative 104-byte limit. The
        // error should be raised *before* hitting `bind(2)`, with a clear
        // message — not an obscure `EINVAL`.
        let long_name = "a".repeat(200);
        let path = std::env::temp_dir().join(format!("{long_name}.sock"));
        let err =
            create_unix_listener(&path, None, false).expect_err("expected path-too-long error");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("is too long"),
            "unexpected error message: {msg}"
        );
    }
}
