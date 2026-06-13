// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module provides a custom [Hyper service](hyper::service::Service).
//!

use hyper::{Request, Response, body::Incoming, service::Service};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use crate::{Error, body, handler::RequestHandler};

#[cfg(feature = "metrics")]
use crate::metrics;

/// It defines the router service which is the main entry point for Hyper Server.
#[derive(Clone)]
pub struct RouterService {
    builder: RequestServiceBuilder,
}

impl RouterService {
    /// Creates a new router service.
    pub fn new(handler: RequestHandler) -> Self {
        Self {
            builder: RequestServiceBuilder::new(handler),
        }
    }

    /// Build a new request service for an accepted connection.
    pub fn build(&self, remote_addr: Option<SocketAddr>) -> RequestService {
        self.builder.build(remote_addr)
    }
}

/// It defines a Hyper service request which delegates a request handler.
pub struct RequestService {
    handler: Arc<RequestHandler>,
    remote_addr: Option<SocketAddr>,
}

impl RequestService {
    fn new(handler: Arc<RequestHandler>, remote_addr: Option<SocketAddr>) -> Self {
        #[cfg(feature = "metrics")]
        metrics::inc_connections();
        Self {
            handler,
            remote_addr,
        }
    }
}

#[cfg(feature = "metrics")]
impl Drop for RequestService {
    fn drop(&mut self) {
        metrics::dec_connections();
    }
}

impl Service<Request<Incoming>> for RequestService {
    type Response = Response<body::Body>;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Response<body::Body>, Error>> + Send + 'static>>;

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
        let handler = self.handler.clone();
        let remote_addr = self.remote_addr;
        Box::pin(async move { handler.handle(&mut req, remote_addr).await })
    }
}

/// It defines a Hyper service request builder.
#[derive(Clone)]
pub struct RequestServiceBuilder {
    handler: Arc<RequestHandler>,
}

impl RequestServiceBuilder {
    /// Initializes a new request service builder.
    pub fn new(handler: RequestHandler) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }

    /// Build a new request service.
    pub fn build(&self, remote_addr: Option<SocketAddr>) -> RequestService {
        RequestService::new(self.handler.clone(), remote_addr)
    }
}
