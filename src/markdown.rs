// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Markdown content negotiation module.
//!
//! This module handles content negotiation for markdown files when the client sends
//! an Accept header that includes text/markdown.

use headers::HeaderMapExt;
use hyper::{Body, Request, Response};
use std::path::Path;

use crate::{
    Error, fs::meta::try_markdown_variant, handler::RequestHandlerOpts, headers_ext::Accept,
};

/// Pre-process a request to check if a markdown variant URI should be used.
/// Returns the modified URI path if a markdown variant exists, None otherwise.
pub(crate) fn pre_process<T>(req: &Request<T>, base_path: &Path, uri_path: &str) -> Option<String> {
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

    tracing::info!("markdown: found variant {:?}", md_path);

    // Convert the markdown file path back to a URI path
    // Strip the base_path and prepend '/'
    md_path
        .strip_prefix(base_path)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| format!("/{}", s))
}

/// Post-process the response to set the correct Content-Type for markdown files.
pub(crate) fn post_process(
    is_markdown_variant: bool,
    opts: &RequestHandlerOpts,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if !is_markdown_variant || !opts.accept_markdown {
        return Ok(resp);
    }

    // Set the Content-Type to text/markdown; charset=utf-8
    if let Ok(content_type) = "text/markdown; charset=utf-8".parse() {
        resp.headers_mut()
            .insert(hyper::header::CONTENT_TYPE, content_type);
    }

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Request};

    #[test]
    fn test_no_accept_header() {
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Without Accept header, should return None (no markdown variant)
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&req, base_path, "/test");

        assert!(result.is_none());
    }

    #[test]
    fn test_accepts_html_only() {
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .header("Accept", "text/html")
            .body(Body::empty())
            .unwrap();

        // With Accept: text/html, should return None (no markdown variant)
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&req, base_path, "/test");

        assert!(result.is_none());
    }

    #[test]
    fn test_accepts_markdown_no_file() {
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .header("Accept", "text/markdown")
            .body(Body::empty())
            .unwrap();

        // With Accept: text/markdown but no file, should return None
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&req, base_path, "/test");

        assert!(result.is_none());
    }
}
