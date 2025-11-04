// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Markdown content negotiation module.
//!
//! This module handles serving markdown files when the client sends an Accept header
//! that includes text/markdown.

use headers::{AcceptRanges, ContentLength, HeaderMapExt};
use hyper::{Body, Request, Response, StatusCode};
use std::path::Path;

use crate::{
    Error, error::anyhow, fs::meta::try_markdown_variant, handler::RequestHandlerOpts,
    headers_ext::Accept, http_ext::MethodExt,
};

/// Pre-process a request to check if a markdown variant should be served instead.
/// This intercepts the request before the normal static file handler.
pub(crate) fn pre_process<T>(
    _opts: &RequestHandlerOpts,
    req: &Request<T>,
    base_path: &Path,
    uri_path: &str,
) -> Option<Result<Response<Body>, Error>> {
    // Only handle GET and HEAD requests
    if !req.method().is_get() && !req.method().is_head() {
        return None;
    }

    // Check if the client accepts markdown
    let accepts_markdown = req
        .headers()
        .typed_get::<Accept>()
        .map(|accept| accept.accepts_markdown())
        .unwrap_or(false);

    if !accepts_markdown {
        return None;
    }

    // Construct the full file path
    let mut file_path = base_path.to_path_buf();
    let sanitized_path = uri_path.trim_start_matches('/');
    if !sanitized_path.is_empty() {
        file_path.push(sanitized_path);
    }

    // Try to find a markdown variant
    let md_path = try_markdown_variant(&file_path)?;

    tracing::info!("markdown: serving {:?}", md_path);

    // Read file based on request method
    let is_head = req.method().is_head();
    let file_result = if is_head {
        std::fs::metadata(&md_path).map(|meta| (Vec::new(), meta.len()))
    } else {
        std::fs::read(&md_path).map(|content| {
            let len = content.len() as u64;
            (content, len)
        })
    };

    match file_result {
        Ok((content, len)) => {
            let body = if is_head {
                Body::empty()
            } else {
                Body::from(content)
            };

            let mut resp = Response::new(body);
            *resp.status_mut() = StatusCode::OK;
            resp.headers_mut().typed_insert(ContentLength(len));
            resp.headers_mut().typed_insert(AcceptRanges::bytes());

            if let Ok(content_type) = "text/markdown; charset=utf-8".parse() {
                resp.headers_mut()
                    .insert(hyper::header::CONTENT_TYPE, content_type);
            }

            Some(Ok(resp))
        }
        Err(err) => {
            tracing::error!("markdown: failed to read {:?}: {}", md_path, err);
            Some(Err(anyhow!("Failed to read markdown file: {}", err)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Request};

    #[test]
    fn test_no_accept_header() {
        let opts = RequestHandlerOpts::default();
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Without Accept header, should return None (no markdown handling)
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&opts, &req, base_path, "/test");

        assert!(result.is_none());
    }

    #[test]
    fn test_accepts_html_only() {
        let opts = RequestHandlerOpts::default();
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .header("Accept", "text/html")
            .body(Body::empty())
            .unwrap();

        // With Accept: text/html, should return None (no markdown handling)
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&opts, &req, base_path, "/test");

        assert!(result.is_none());
    }

    #[test]
    fn test_post_method_ignored() {
        let opts = RequestHandlerOpts::default();
        let req = Request::builder()
            .method("POST")
            .uri("/test")
            .header("Accept", "text/markdown")
            .body(Body::empty())
            .unwrap();

        // POST method should be ignored
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&opts, &req, base_path, "/test");

        assert!(result.is_none());
    }
}
