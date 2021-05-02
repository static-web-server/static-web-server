use hyper::{Body, Request, Response};
use std::path::Path;

use crate::{compression, static_files};
use crate::{error::Result, error_page};

/// Main server request handler.
pub async fn handle_request(base: &Path, req: Request<Body>) -> Result<Response<Body>> {
    let headers = req.headers();
    let method = req.method();

    match static_files::handle_request(method, headers, base, req.uri().path()).await {
        Ok(resp) => compression::auto(method, headers, resp),
        Err(status) => error_page::get_error_response(method, &status),
    }
}
