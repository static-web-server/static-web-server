// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Fallback page module useful for a custom page default.
//!

use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Request, Response, StatusCode};
use mime_guess::mime;
use std::path::Path;

use crate::{handler::RequestHandlerOpts, helpers, http_ext::MethodExt, Error};

/// Initializes fallback page processing
pub(crate) fn init(path: &Path, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.page_fallback = helpers::read_bytes_default(path);

    server_info!(
        "fallback page: enabled={}, value=\"{}\"",
        !handler_opts.page_fallback.is_empty(),
        path.to_string_lossy()
    );
}

/// Replace 404 Not Found by the configured fallback page
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    Ok(
        if req.method().is_get()
            && resp.status() == StatusCode::NOT_FOUND
            && !opts.page_fallback.is_empty()
        {
            fallback_response(&opts.page_fallback)
        } else {
            resp
        },
    )
}

/// Checks if a fallback response can be generated, i.e. if it is a `GET` request
/// that would result in a `404` error and a fallback page is configured.
/// If a response can be generated then is returned otherwise `None`.
pub fn fallback_response(page_fallback: &[u8]) -> Response<Body> {
    let body = Body::from(page_fallback.to_owned());
    let len = page_fallback.len() as u64;

    let mut resp = Response::new(body);
    *resp.status_mut() = StatusCode::OK;

    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    resp
}

#[cfg(test)]
mod tests {
    use super::post_process;
    use crate::{error_page, handler::RequestHandlerOpts, Error};
    use hyper::{Body, Method, Request, Response, StatusCode, Uri};
    use std::path::PathBuf;

    fn make_request(method: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri("/")
            .body(Body::empty())
            .unwrap()
    }

    fn make_response(status: &StatusCode) -> Response<Body> {
        error_page::error_response(
            &Uri::try_from("/").unwrap(),
            &Method::GET,
            status,
            &PathBuf::new(),
            &PathBuf::new(),
        )
        .unwrap()
    }

    #[test]
    fn test_success_code() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            page_fallback: vec![1, 2, 3],
            ..Default::default()
        };
        let req = make_request("GET");
        let resp = make_response(&StatusCode::OK);

        let resp = post_process(&opts, &req, resp)?;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_ne!(
            resp.headers()
                .get("Content-Length")
                .map(|v| v.to_str().unwrap())
                .unwrap_or("3"),
            "3"
        );

        Ok(())
    }

    #[test]
    fn test_wrong_error() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            page_fallback: vec![1, 2, 3],
            ..Default::default()
        };
        let req = make_request("GET");
        let resp = make_response(&StatusCode::INTERNAL_SERVER_ERROR);

        let resp = post_process(&opts, &req, resp)?;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_ne!(
            resp.headers()
                .get("Content-Length")
                .map(|v| v.to_str().unwrap())
                .unwrap_or("3"),
            "3"
        );

        Ok(())
    }

    #[test]
    fn test_wrong_method() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            page_fallback: vec![1, 2, 3],
            ..Default::default()
        };
        let req = make_request("POST");
        let resp = make_response(&StatusCode::NOT_FOUND);

        let resp = post_process(&opts, &req, resp)?;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(
            resp.headers()
                .get("Content-Length")
                .map(|v| v.to_str().unwrap())
                .unwrap_or("3"),
            "3"
        );

        Ok(())
    }

    #[test]
    fn test_unconfigured() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            page_fallback: Vec::new(),
            ..Default::default()
        };
        let req = make_request("GET");
        let resp = make_response(&StatusCode::NOT_FOUND);

        let resp = post_process(&opts, &req, resp)?;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[test]
    fn test_fallback() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            page_fallback: vec![1, 2, 3],
            ..Default::default()
        };
        let req = make_request("GET");
        let resp = make_response(&StatusCode::NOT_FOUND);

        let resp = post_process(&opts, &req, resp)?;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Length")
                .map(|v| v.to_str().unwrap())
                .unwrap_or("3"),
            "3"
        );

        Ok(())
    }
}
