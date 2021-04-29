use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Request, Response};
use std::path::Path;

use crate::error::Result;
use crate::fs;

/// Main server request entry point.
pub async fn handle(base: &Path, req: Request<Body>) -> Result<Response<Body>> {
    let path = req.uri().path();
    let resp = fs::handle_request(base, req.headers(), path).await;
    match resp {
        Ok(resp) => Ok(resp),
        Err(status) => {
            let method = req.method();
            tracing::warn!(method = ?method, status = status.as_u16(), error = ?status.to_string());

            let mut body = Body::empty();
            let mut len = 0_u64;
            if method == hyper::Method::GET {
                let content = format!(
                    "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
                    status, status
                );
                len = content.len() as u64;
                body = Body::from(content)
            }

            let mut resp = Response::new(body);
            *resp.status_mut() = status;
            resp.headers_mut().typed_insert(ContentLength(len));
            resp.headers_mut().typed_insert(ContentType::html());
            resp.headers_mut().typed_insert(AcceptRanges::bytes());

            Ok(resp)
        }
    }
}
