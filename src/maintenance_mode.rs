// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Provides maintenance mode functionality.
//!

use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Method, Request, Response, StatusCode};
use mime_guess::mime;
use std::path::{Path, PathBuf};

use crate::{handler::RequestHandlerOpts, helpers, http_ext::MethodExt, Error, Result};

const DEFAULT_BODY_CONTENT: &str = "The server is in maintenance mode.";

/// Initializes maintenance mode handling
pub(crate) fn init(
    maintenance_mode: bool,
    maintenance_mode_status: StatusCode,
    maintenance_mode_file: PathBuf,
    handler_opts: &mut RequestHandlerOpts,
) {
    handler_opts.maintenance_mode = maintenance_mode;
    handler_opts.maintenance_mode_status = maintenance_mode_status;
    handler_opts.maintenance_mode_file = maintenance_mode_file;
    server_info!(
        "maintenance mode: enabled={}",
        handler_opts.maintenance_mode
    );
    server_info!(
        "maintenance mode status: {}",
        handler_opts.maintenance_mode_status.as_str()
    );
    server_info!(
        "maintenance mode file: \"{}\"",
        handler_opts.maintenance_mode_file.display()
    );
}

/// Produces maintenance mode response if necessary
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    if opts.maintenance_mode {
        Some(get_response(
            req.method(),
            &opts.maintenance_mode_status,
            &opts.maintenance_mode_file,
        ))
    } else {
        None
    }
}

/// Get the a server maintenance mode response.
pub fn get_response(
    method: &Method,
    status_code: &StatusCode,
    file_path: &Path,
) -> Result<Response<Body>> {
    tracing::debug!("server has entered into maintenance mode");
    tracing::debug!("maintenance mode file path to use: {}", file_path.display());

    let body_content = if file_path.is_file() {
        String::from_utf8_lossy(&helpers::read_bytes_default(file_path))
            .trim()
            .to_owned()
    } else {
        tracing::debug!(
            "maintenance mode file path not found or not a regular file, using a default message"
        );
        format!("<html><head><title>{status_code}</title></head><body><center><h1>{DEFAULT_BODY_CONTENT}</h1></center></body></html>")
    };

    let mut body = Body::empty();
    let len = body_content.len() as u64;

    if !method.is_head() {
        body = Body::from(body_content)
    }

    let mut resp = Response::new(body);
    *resp.status_mut() = *status_code;
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::pre_process;
    use crate::{handler::RequestHandlerOpts, Error};
    use hyper::{Body, Request, Response, StatusCode};

    fn make_request() -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap()
    }

    fn get_status(result: Option<Result<Response<Body>, Error>>) -> Option<StatusCode> {
        if let Some(Ok(response)) = result {
            Some(response.status())
        } else {
            None
        }
    }

    #[test]
    fn test_maintenance_disabled() {
        assert!(pre_process(
            &RequestHandlerOpts {
                maintenance_mode: false,
                ..Default::default()
            },
            &make_request()
        )
        .is_none());
    }

    #[test]
    fn test_maintenance_default() {
        assert_eq!(
            get_status(pre_process(
                &RequestHandlerOpts {
                    maintenance_mode: true,
                    ..Default::default()
                },
                &make_request()
            )),
            Some(StatusCode::SERVICE_UNAVAILABLE)
        );
    }

    #[test]
    fn test_maintenance_custom_status() {
        assert_eq!(
            get_status(pre_process(
                &RequestHandlerOpts {
                    maintenance_mode: true,
                    maintenance_mode_status: StatusCode::IM_A_TEAPOT,
                    ..Default::default()
                },
                &make_request()
            )),
            Some(StatusCode::IM_A_TEAPOT)
        );
    }
}
