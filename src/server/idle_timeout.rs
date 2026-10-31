// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Connection IO wrapper that fails once a connection has made no read or write progress for a while,
//! so that Hyper closes it.

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::time::{Instant, Sleep};

/// Connection IO that fails with [`io::ErrorKind::TimedOut`]
/// once no read or write has made progress for the idle timeout.
///
/// The timeout is only checked while a read or write is pending.
/// Hyper usually keeps a read pending while a response is in progress too,
/// so a response that doesn't start, or whose body stalls, within the timeout fails as well.
/// Writes only make progress once the kernel has room in the send buffer,
/// so a client that reads very slowly can also reach the timeout.
/// Over TLS only decrypted bytes count, so the handshake must also complete within the timeout.
pub(crate) struct IdleTimeout<T> {
    inner: T,
    /// The idle timer, or `None` when the timeout is disabled.
    timer: Option<Timer>,
}

/// Idle timer state of an [`IdleTimeout`] connection.
struct Timer {
    /// Time without progress after which the connection fails.
    timeout: Duration,
    /// Time of the last read or write progress.
    last_progress: Instant,
    /// Wake-up for the earliest possible expiry.
    /// It's only moved forward when it fires, so progress doesn't touch the timer wheel.
    sleep: Pin<Box<Sleep>>,
}

impl<T> IdleTimeout<T> {
    /// Wrap `inner`, failing it after `timeout` without progress, or never if `None`.
    pub(crate) fn new(inner: T, timeout: Option<Duration>) -> Self {
        Self {
            inner,
            timer: timeout.map(|timeout| Timer {
                timeout,
                last_progress: Instant::now(),
                sleep: Box::pin(tokio::time::sleep(timeout)),
            }),
        }
    }

    /// Record progress if bytes were read or written,
    /// and fail a pending operation once the connection has been idle for the timeout.
    ///
    /// Only transferred bytes count: an idle HTTP/2 connection completes a flush on every poll.
    fn check<R>(
        &mut self,
        cx: &mut Context<'_>,
        poll: Poll<io::Result<R>>,
        progressed: bool,
    ) -> Poll<io::Result<R>> {
        let Some(timer) = &mut self.timer else {
            return poll;
        };
        if progressed {
            timer.last_progress = Instant::now();
        }
        if poll.is_ready() {
            return poll;
        }
        let deadline = timer.last_progress + timer.timeout;
        while timer.sleep.as_mut().poll(cx).is_ready() {
            if timer.sleep.deadline() >= deadline {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "connection idle timeout",
                )));
            }
            timer.sleep.as_mut().reset(deadline);
        }
        Poll::Pending
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for IdleTimeout<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let filled = buf.filled().len();
        let poll = Pin::new(&mut this.inner).poll_read(cx, buf);
        let progressed = buf.filled().len() > filled;
        this.check(cx, poll, progressed)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for IdleTimeout<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let poll = Pin::new(&mut this.inner).poll_write(cx, buf);
        let progressed = matches!(poll, Poll::Ready(Ok(n)) if n > 0);
        this.check(cx, poll, progressed)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let poll = Pin::new(&mut this.inner).poll_write_vectored(cx, bufs);
        let progressed = matches!(poll, Poll::Ready(Ok(n)) if n > 0);
        this.check(cx, poll, progressed)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let poll = Pin::new(&mut this.inner).poll_flush(cx);
        this.check(cx, poll, false)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let poll = Pin::new(&mut this.inner).poll_shutdown(cx);
        this.check(cx, poll, false)
    }
}

#[cfg(test)]
mod tests {
    use super::IdleTimeout;
    use std::io::IoSlice;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

    const TIMEOUT: Duration = Duration::from_millis(200);

    #[tokio::test]
    async fn read_fails_after_idle_timeout() {
        let (server, _client) = duplex(64);
        let mut io = IdleTimeout::new(server, Some(TIMEOUT));
        let mut buf = [0u8; 1];
        let err = tokio::time::timeout(Duration::from_secs(5), io.read(&mut buf))
            .await
            .expect("idle read did not time out")
            .expect_err("idle read succeeded");
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    }

    #[tokio::test]
    async fn write_fails_after_idle_timeout() {
        // The peer never reads, so the write blocks once the buffer is full.
        let (server, _client) = duplex(64);
        let mut io = IdleTimeout::new(server, Some(TIMEOUT));
        let err = tokio::time::timeout(Duration::from_secs(5), io.write_all(&[0u8; 1024]))
            .await
            .expect("blocked write did not time out")
            .expect_err("blocked write succeeded");
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    }

    #[tokio::test]
    async fn flush_is_not_progress() {
        let (server, _client) = duplex(64);
        let mut io = IdleTimeout::new(server, Some(TIMEOUT));
        let mut buf = [0u8; 1];
        let start = std::time::Instant::now();
        // Flush between reads, like Hyper's HTTP/2 connection does whenever it's polled.
        let err = loop {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "flushes kept the connection open"
            );
            io.flush().await.expect("flush");
            if let Ok(read) = tokio::time::timeout(TIMEOUT / 4, io.read(&mut buf)).await {
                break read.expect_err("idle read succeeded");
            }
        };
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    }

    #[tokio::test]
    async fn read_progress_restarts_idle_timeout() {
        let (server, mut client) = duplex(64);
        let mut io = IdleTimeout::new(server, Some(TIMEOUT));
        let mut buf = [0u8; 4];
        // One byte every half timeout: 2 timeouts in total, never a full one without progress.
        let write = async {
            for _ in 0..4 {
                tokio::time::sleep(TIMEOUT / 2).await;
                client.write_all(b"x").await.expect("client write");
            }
        };
        let (read, ()) = tokio::join!(io.read_exact(&mut buf), write);
        read.expect("read with progress failed");
    }

    #[tokio::test]
    async fn write_progress_restarts_idle_timeout() {
        assert_write_progress(false).await;
    }

    #[tokio::test]
    async fn vectored_write_progress_restarts_idle_timeout() {
        assert_write_progress(true).await;
    }

    /// Write to a peer that reads one byte every half timeout:
    /// 2 timeouts in total, never a full one without progress.
    async fn assert_write_progress(vectored: bool) {
        let (server, mut client) = duplex(1);
        let mut io = IdleTimeout::new(server, Some(TIMEOUT));
        let read = async {
            let mut buf = [0u8; 1];
            for _ in 0..4 {
                tokio::time::sleep(TIMEOUT / 2).await;
                if client.read_exact(&mut buf).await.is_err() {
                    break;
                }
            }
        };
        // Move the IO into the write, so that a failed write closes it and ends the reads.
        let write = async move {
            // One byte fills the buffer and the peer reads the other 4.
            let data = [0u8; 5];
            let mut written = 0;
            while written < data.len() {
                written += if vectored {
                    io.write_vectored(&[IoSlice::new(&data[written..])]).await?
                } else {
                    io.write(&data[written..]).await?
                };
            }
            Ok::<_, std::io::Error>(())
        };
        let (write, ()) = tokio::join!(write, read);
        write.expect("write with progress failed");
    }

    #[tokio::test]
    async fn disabled_timeout_never_fails() {
        let (server, _client) = duplex(64);
        let mut io = IdleTimeout::new(server, None);
        let mut buf = [0u8; 1];
        assert!(
            tokio::time::timeout(TIMEOUT * 3, io.read(&mut buf))
                .await
                .is_err(),
            "read without a timeout finished"
        );
    }
}
