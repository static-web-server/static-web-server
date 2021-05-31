use hyper::{service::Service, Body, Request, Response};
use std::convert::Infallible;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::handler::RequestHandler;
use crate::Error;

/// It defines the router service which is the main entry point for Hyper Server.
pub struct RouterService {
    builder: RequestServiceBuilder,
}

impl RouterService {
    pub fn new(handler: RequestHandler) -> Self {
        Self {
            builder: RequestServiceBuilder::new(handler),
        }
    }
}

impl<T> Service<T> for RouterService {
    type Response = RequestService;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        ready(Ok(self.builder.build()))
    }
}

/// It defines a Hyper service request which delegates a request handler.
pub struct RequestService {
    handler: Arc<RequestHandler>,
}

impl Service<Request<Body>> for RequestService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let handler = self.handler.clone();
        Box::pin(async move { handler.handle(&mut req).await })
    }
}

/// It defines a Hyper service request builder.
pub struct RequestServiceBuilder {
    handler: Arc<RequestHandler>,
}

impl RequestServiceBuilder {
    pub fn new(handler: RequestHandler) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }

    pub fn build(&self) -> RequestService {
        RequestService {
            handler: self.handler.clone(),
        }
    }
}
