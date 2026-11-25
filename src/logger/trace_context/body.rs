// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The response body wrapper that keeps the trace context span while hyper sends the body.

use bytes::Bytes;
use http_body::{Frame, SizeHint};
use http_body_util::BodyExt;
use hyper::Response;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::Span;

use crate::body::Body;

/// Makes the response body enter `span` when it is polled and dropped, so the log lines written
/// while the body is sent, such as the access log line, keep the trace context fields. hyper
/// polls and drops the body after the request handler future, so outside of its span.
///
/// The response is returned as is when `span` is disabled.
pub(crate) fn wrap(span: Span, resp: Response<Body>) -> Response<Body> {
    if span.is_none() {
        return resp;
    }
    resp.map(|inner| SpannedBody { inner, span }.boxed())
}

/// A response body that is polled and dropped inside a span.
struct SpannedBody {
    inner: Body,
    span: Span,
}

impl http_body::Body for SpannedBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, io::Error>>> {
        let this = self.get_mut();
        let _entered = this.span.enter();
        Pin::new(&mut this.inner).poll_frame(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

impl Drop for SpannedBody {
    fn drop(&mut self) {
        let _entered = self.span.enter();
        // An empty body does not allocate, so the inner one is dropped inside the span
        drop(std::mem::replace(&mut self.inner, crate::body::empty()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body::Body as _;
    use std::sync::{Arc, Mutex};

    /// A body that stores the spans it is polled and dropped in.
    #[derive(Default, Clone)]
    struct SpanProbe {
        polled_in: Arc<Mutex<Option<tracing::Id>>>,
        dropped_in: Arc<Mutex<Option<tracing::Id>>>,
    }

    impl http_body::Body for SpanProbe {
        type Data = Bytes;
        type Error = io::Error;

        fn poll_frame(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Bytes>, io::Error>>> {
            *self.polled_in.lock().unwrap() = Span::current().id();
            Poll::Ready(None)
        }
    }

    impl Drop for SpanProbe {
        fn drop(&mut self) {
            *self.dropped_in.lock().unwrap() = Span::current().id();
        }
    }

    #[tokio::test]
    async fn wrapped_body_is_polled_and_dropped_in_the_span() {
        crate::logger::register_test_subscribers();
        let _guard = tracing::subscriber::set_default(tracing_subscriber::registry());

        let span = tracing::error_span!("request");
        let probe = SpanProbe::default();
        let resp = wrap(span.clone(), Response::new(probe.clone().boxed()));

        // The locks are released before the asserts: a failed one must not poison the mutex
        // that the probe locks when it is dropped, which would abort the test binary
        let mut body = resp.into_body();
        assert!(body.frame().await.is_none());
        let polled_in = probe.polled_in.lock().unwrap().clone();
        assert_eq!(polled_in, span.id());
        drop(body);
        let dropped_in = probe.dropped_in.lock().unwrap().clone();
        assert_eq!(dropped_in, span.id());
    }

    /// The wrapper passes the frames, the size and the end of the body through, so hyper still
    /// sends the exact `Content-Length`, and none for a response without a body over HTTP/2.
    #[tokio::test]
    async fn wrapped_body_keeps_the_frames_and_the_size() {
        crate::logger::register_test_subscribers();
        let _guard = tracing::subscriber::set_default(tracing_subscriber::registry());

        let span = tracing::error_span!("request");
        assert!(!span.is_none(), "the body must be wrapped");
        let resp = wrap(span.clone(), Response::new(crate::body::full("abc")));
        let body = resp.into_body();
        assert!(!body.is_end_stream());
        assert_eq!(body.size_hint().exact(), Some(3));
        assert_eq!(body.collect().await.unwrap().to_bytes(), "abc");

        let resp = wrap(span, Response::new(crate::body::empty()));
        assert!(resp.body().is_end_stream());
    }
}
