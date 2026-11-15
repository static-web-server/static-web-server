// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module provides a custom [Hyper service](hyper::service::Service).
//!

use hyper::{Body, Request, Response, service::Service};
use std::convert::Infallible;
use std::future::{Future, Ready, ready};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::{Error, handler::RequestHandler, security_headers, transport::Transport};

#[cfg(feature = "metrics")]
use crate::metrics;

/// It defines the router service which is the main entry point for Hyper Server.
pub struct RouterService {
    builder: RequestServiceBuilder,
}

impl RouterService {
    /// Creates a new router service.
    ///
    /// Requests are handled as received over plain HTTP, so the security headers
    /// don't include `Strict-Transport-Security`. When serving it behind your own
    /// TLS layer, add that header with a custom header instead.
    pub fn new(handler: RequestHandler) -> Self {
        Self::with_tls(handler, false)
    }

    /// Creates a new router service whose connections use TLS when `tls` is `true`.
    pub(crate) fn with_tls(handler: RequestHandler, tls: bool) -> Self {
        Self {
            builder: RequestServiceBuilder::with_tls(handler, tls),
        }
    }
}

impl<T: Transport + Send + 'static> Service<&T> for RouterService {
    type Response = RequestService;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, conn: &T) -> Self::Future {
        ready(Ok(self.builder.build(conn.remote_addr())))
    }
}

/// It defines a Hyper service request which delegates a request handler.
pub struct RequestService {
    handler: Arc<RequestHandler>,
    remote_addr: Option<SocketAddr>,
    tls: bool,
}

impl RequestService {
    fn new(handler: Arc<RequestHandler>, remote_addr: Option<SocketAddr>, tls: bool) -> Self {
        #[cfg(feature = "metrics")]
        metrics::inc_connections();
        Self {
            handler,
            remote_addr,
            tls,
        }
    }
}

#[cfg(feature = "metrics")]
impl Drop for RequestService {
    fn drop(&mut self) {
        metrics::dec_connections();
    }
}

impl Service<Request<Body>> for RequestService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if self.tls {
            req.extensions_mut().insert(security_headers::TlsConnection);
        }
        let handler = self.handler.clone();
        let remote_addr = self.remote_addr;
        Box::pin(async move { handler.handle(&mut req, remote_addr).await })
    }
}

/// It defines a Hyper service request builder.
pub struct RequestServiceBuilder {
    handler: Arc<RequestHandler>,
    tls: bool,
}

impl RequestServiceBuilder {
    /// Initializes a new request service builder.
    ///
    /// Requests are handled as received over plain HTTP, see [`RouterService::new`].
    pub fn new(handler: RequestHandler) -> Self {
        Self::with_tls(handler, false)
    }

    /// Initializes a new request service builder whose connections use TLS when `tls` is `true`.
    pub(crate) fn with_tls(handler: RequestHandler, tls: bool) -> Self {
        Self {
            handler: Arc::new(handler),
            tls,
        }
    }

    /// Build a new request service.
    pub fn build(&self, remote_addr: Option<SocketAddr>) -> RequestService {
        RequestService::new(self.handler.clone(), remote_addr, self.tls)
    }
}

#[cfg(test)]
mod tests {
    use hyper::{Body, Request, Response, service::Service};

    use super::RequestServiceBuilder;
    use crate::{
        Settings,
        testing::fixtures::{fixture_req_handler, fixture_req_handler_opts},
    };

    async fn get_index(security_headers: &str, tls: bool) -> Response<Body> {
        let opts = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "tests/fixtures/public",
                security_headers,
            ],
        )
        .unwrap();
        let handler = fixture_req_handler(fixture_req_handler_opts(opts.general, opts.advanced));
        let mut service = RequestServiceBuilder::with_tls(handler, tls).build(None);

        let req = Request::get("http://localhost/index.htm")
            .body(Body::empty())
            .unwrap();
        let resp = service.call(req).await.unwrap();
        assert_eq!(resp.status(), 200);
        resp
    }

    #[tokio::test]
    async fn sends_hsts_only_over_tls() {
        for tls in [false, true] {
            let resp = get_index("--security-headers=true", tls).await;
            assert!(resp.headers().contains_key("x-frame-options"));
            assert_eq!(
                resp.headers().contains_key("strict-transport-security"),
                tls,
                "tls={tls}"
            );
        }
    }

    #[tokio::test]
    async fn no_security_headers_when_disabled() {
        for tls in [false, true] {
            let resp = get_index("--security-headers=false", tls).await;
            assert!(!resp.headers().contains_key("x-frame-options"));
            assert!(!resp.headers().contains_key("strict-transport-security"));
        }
    }

    #[tokio::test]
    async fn plain_builder_sends_no_hsts() {
        let opts = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "tests/fixtures/public",
                "--security-headers=true",
            ],
        )
        .unwrap();
        let handler = fixture_req_handler(fixture_req_handler_opts(opts.general, opts.advanced));
        let mut service = RequestServiceBuilder::new(handler).build(None);

        let req = Request::get("http://localhost/index.htm")
            .body(Body::empty())
            .unwrap();
        let resp = service.call(req).await.unwrap();
        assert!(resp.headers().contains_key("x-frame-options"));
        assert!(!resp.headers().contains_key("strict-transport-security"));
    }
}
