use hyper::{Body, Request, Response};
use std::path::Path;

use crate::static_files;
use crate::{error::Result, error_page};

/// Main server request handler.
pub async fn handle_request(base: &Path, req: Request<Body>) -> Result<Response<Body>> {
    match static_files::handle_request(base, req.headers(), req.uri().path()).await {
        Ok(resp) => Ok(resp),
        Err(status) => error_page::get_error_response(req.method(), &status),
    }
}
