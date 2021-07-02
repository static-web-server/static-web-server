use http::StatusCode;
use hyper::{Body, Request, Response};
use std::{future::Future, path::PathBuf, sync::Arc};

use crate::{compression, control_headers, cors, error_page, security_headers, static_files};
use crate::{Error, Result};

/// It defines options for a request handler.
pub struct RequestHandlerOpts {
    pub root_dir: PathBuf,
    pub compression: bool,
    pub dir_listing: bool,
    pub cors: Option<Arc<cors::Configured>>,
    pub security_headers: bool,
}

/// It defines the main request handler used by the Hyper service request.
pub struct RequestHandler {
    pub opts: RequestHandlerOpts,
}

impl RequestHandler {
    /// Main entry point for incoming requests.
    pub fn handle<'a>(
        &'a self,
        req: &'a mut Request<Body>,
    ) -> impl Future<Output = Result<Response<Body>, Error>> + Send + 'a {
        let method = req.method();
        let headers = req.headers();

        let root_dir = self.opts.root_dir.as_path();
        let uri_path = req.uri().path();
        let dir_listing = self.opts.dir_listing;

        async move {
            // CORS
            if self.opts.cors.is_some() {
                let cors = self.opts.cors.as_ref().unwrap();
                match cors.check_request(method, headers) {
                    Ok(r) => {
                        tracing::debug!("cors ok: {:?}", r);
                    }
                    Err(e) => {
                        tracing::debug!("cors error kind: {:?}", e);
                        return error_page::error_response(method, &StatusCode::FORBIDDEN);
                    }
                };
            }

            // Static files
            match static_files::handle(method, headers, root_dir, uri_path, dir_listing).await {
                Ok(mut resp) => {
                    // Auto compression based on the `Accept-Encoding` header
                    if self.opts.compression {
                        resp = match compression::auto(method, headers, resp) {
                            Ok(res) => res,
                            Err(err) => {
                                tracing::debug!("error during body compression: {:?}", err);
                                return error_page::error_response(
                                    method,
                                    &StatusCode::INTERNAL_SERVER_ERROR,
                                );
                            }
                        };
                    }

                    // Append `Cache-Control` headers for web assets
                    let ext = uri_path.to_lowercase();
                    control_headers::append_headers(&ext, &mut resp);

                    // Append security headers
                    if self.opts.security_headers {
                        security_headers::append_headers(&mut resp);
                    }

                    Ok(resp)
                }
                Err(status) => error_page::error_response(method, &status),
            }
        }
    }
}
