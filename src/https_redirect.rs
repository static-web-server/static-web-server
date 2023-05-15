//! Module to redirect HTTP requests to HTTPS.
//!

use headers::{HeaderMapExt, Host};
use hyper::{header::LOCATION, Body, Request, Response, StatusCode};

use crate::Result;

/// It redirects all requests from HTTP to HTTPS.
pub async fn redirect_to_https(req: Request<Body>, port: u16) -> Result<Response<Body>> {
    if let Some(ref host) = req.headers().typed_get::<Host>() {
        let url = format!("https://{}:{}{}", host.hostname(), port, req.uri());
        tracing::debug!("https redirect to {}", url);

        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
        resp.headers_mut().insert(LOCATION, url.parse().unwrap());
        return Ok(resp);
    }

    bail!("error: host was not determined!")
}
