// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module provides a custom [Hyper service](hyper::service::Service).
//!

use hyper::{service::Service, Body, Request, Response};
use std::convert::Infallible;
use std::future::{ready, Future, Ready};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::{handler::RequestHandler, transport::Transport, Error};

/// It defines the router service which is the main entry point for Hyper Server.
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
}

impl Service<Request<Body>> for RequestService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let handler = self.handler.clone();
        let remote_addr = self.remote_addr;
        Box::pin(async move { handler.handle(&mut req, remote_addr).await })
    }
}

/// It defines a Hyper service request builder.
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
        RequestService {
            handler: self.handler.clone(),
            remote_addr,
        }
    }
}
