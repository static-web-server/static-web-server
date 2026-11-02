// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Fallback page module useful for a custom page default.
//!

use headers::HeaderMapExt;
use hyper::{Request, Response, StatusCode};
use std::path::Path;

use crate::body::Body;
use crate::error_page::build_html_response;
use crate::exts::headers::Accept;
use crate::{Error, exts::http::MethodExt, handler::RequestHandlerOpts, helpers};

/// Initializes fallback page processing
pub(crate) fn init(file_path: &Path, handler_opts: &mut RequestHandlerOpts) {
    let found = file_path.is_file();
    if found {
        handler_opts.page_fallback = helpers::read_text_default(file_path).into_bytes();
    } else {
        tracing::debug!("fallback page path not found or not a regular file");
    }

    tracing::info!(
        "fallback page: enabled={}, value=\"{}\"",
        found,
        file_path.display()
    );
}

/// Replace 404 Not Found by the configured fallback page
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    Ok(if can_serve(opts, req, resp.status()) {
        fallback_response(&opts.page_fallback)
    } else {
        resp
    })
}

/// Returns whether the request and status will be replaced by a fallback page.
pub(crate) fn can_serve<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    status: StatusCode,
) -> bool {
    req.method().is_get()
        && status == StatusCode::NOT_FOUND
        && !opts.page_fallback.is_empty()
        && !is_missing_asset(req)
}

/// Returns whether the request is for a missing asset (like `/favicon.ico`) rather than a page.
///
/// That is a path with a known non-HTML file extension, sent by a client that did not
/// explicitly ask for HTML. A wildcard like `*/*` does not count as asking for HTML,
/// because browsers append it to the `Accept` header of image and script requests.
fn is_missing_asset<T>(req: &Request<T>) -> bool {
    let names_non_html_file = mime_guess::from_path(req.uri().path())
        .first()
        .is_some_and(|mime| !matches!(mime.essence_str(), "text/html" | "application/xhtml+xml"));

    names_non_html_file
        && !req.headers().typed_get::<Accept>().is_some_and(|accept| {
            accept.accepts("text/html") || accept.accepts("application/xhtml+xml")
        })
}

/// Checks if a fallback response can be generated, i.e. if it is a `GET` request
/// that would result in a `404` error and a fallback page is configured.
/// If a response can be generated then is returned otherwise `None`.
pub fn fallback_response(page_fallback: &[u8]) -> Response<Body> {
    build_html_response(page_fallback.to_owned(), StatusCode::OK, None)
}

#[cfg(test)]
mod tests {
    use super::post_process;
    use crate::body::Body;
    use crate::{Error, error_page, handler::RequestHandlerOpts};
    use hyper::{Method, Request, Response, StatusCode, Uri};
    use std::path::PathBuf;

    fn make_request(method: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri("/")
            .body(crate::body::empty())
            .unwrap()
    }

    fn make_request_to(uri: &str, accept: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method("GET").uri(uri);
        if let Some(accept) = accept {
            builder = builder.header("Accept", accept);
        }
        builder.body(crate::body::empty()).unwrap()
    }

    fn fallback_status(uri: &str, accept: Option<&str>) -> StatusCode {
        let opts = RequestHandlerOpts {
            page_fallback: vec![1, 2, 3],
            ..Default::default()
        };
        let req = make_request_to(uri, accept);
        let resp = make_response(&StatusCode::NOT_FOUND);
        post_process(&opts, &req, resp).unwrap().status()
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

    #[test]
    fn test_missing_asset_keeps_not_found() {
        // Browsers ask for the favicon with an image `Accept` list that ends in `*/*`.
        let accept = "image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5";
        assert_eq!(
            fallback_status("/favicon.ico", Some(accept)),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            fallback_status("/assets/app.js", Some("*/*")),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            fallback_status("/img/logo.png", None),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_html_navigation_still_falls_back() {
        let accept = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
        // A client-side route.
        assert_eq!(fallback_status("/users/42", Some(accept)), StatusCode::OK);
        assert_eq!(fallback_status("/users/42", Some("*/*")), StatusCode::OK);
        assert_eq!(fallback_status("/users/42", None), StatusCode::OK);
        // A route that happens to end in a file-like segment, requested as a page.
        assert_eq!(
            fallback_status("/files/report.pdf", Some(accept)),
            StatusCode::OK
        );
    }

    #[test]
    fn test_html_and_unknown_extensions_still_fall_back() {
        assert_eq!(fallback_status("/missing.html", None), StatusCode::OK);
        assert_eq!(
            fallback_status("/user/john.doe", Some("*/*")),
            StatusCode::OK
        );
    }

    #[test]
    fn test_zero_quality_html_does_not_count_as_requested() {
        assert_eq!(
            fallback_status("/favicon.ico", Some("text/html;q=0, */*")),
            StatusCode::NOT_FOUND
        );
    }
}
