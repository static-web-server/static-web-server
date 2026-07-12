// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Markdown content negotiation module.
//!
//! This module handles content negotiation for markdown files when the client sends
//! an Accept header that includes text/markdown.

use headers::HeaderMapExt;
use hyper::{Request, Response};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use std::path::Path;

use crate::body::Body;
use crate::fs::path::{PathExt, sanitize_path};
use crate::{
    Error, exts::headers::Accept, fs::meta::try_markdown_variant, handler::RequestHandlerOpts,
};

/// Escape set used when converting a resolved file path back into a URI path.
///
/// The returned URI is percent-decoded again by `sanitize_path` in the
/// static-files handler, so `%` must be escaped to keep the encode/decode
/// round-trip lossless for file names that contain it.
const URI_PATH_ESCAPE: &AsciiSet = &CONTROLS.add(b'%');

/// Pre-process a request to check if a markdown variant URI should be used.
///
/// The URI path is sanitized with the same `sanitize_path` used by the
/// static-files handler before any filesystem access, so the `metadata()`
/// probes issued by `try_markdown_variant` can never reach outside
/// `base_path`. On any failure the function returns `None` and the request
/// falls through to the static-files handler (fail closed, 404).
///
/// Symlink policy is intentionally *not* enforced here: the rewritten URI is
/// re-sanitized and passed through containment, symlink and hidden-file
/// checks in `static_files::handle`, which is the authoritative gate.
pub(crate) fn pre_process<T>(
    req: &Request<T>,
    base_path: &Path,
    uri_path: &str,
    include_hidden: bool,
) -> Option<String> {
    let accepts_markdown = req
        .headers()
        .typed_get::<Accept>()
        .is_some_and(|accept| accept.accepts_markdown());
    if !accepts_markdown {
        return None;
    }

    // SECURITY: percent-decode and strip traversal components before
    // touching the filesystem.
    let file_path = sanitize_path(base_path, uri_path).ok()?;

    // `sanitize_path` only pushes `Normal` components onto `base_path`,
    // so this prefix strip cannot fail in practice.
    let relative = file_path.strip_prefix(base_path).ok()?;

    // Hidden-file policy check
    if !include_hidden && relative.is_hidden() {
        tracing::debug!("markdown: skipping hidden path {:?}", relative);
        return None;
    }

    let md_path = try_markdown_variant(&file_path)?;
    tracing::debug!("markdown: found variant {:?}", md_path);

    // Convert the variant path back into a URI path, escaping `%` so the
    // static-files handler's re-decode resolves to the same file.
    let relative_md = md_path.strip_prefix(base_path).ok()?.to_str()?;
    Some(format!(
        "/{}",
        utf8_percent_encode(relative_md, URI_PATH_ESCAPE)
    ))
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

    resp.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("text/markdown; charset=utf-8"),
    );

    Ok(resp)
}

// Property-based regression test for `pre_process`
//
// These properties encode the security invariants of markdown content
// negotiation: regardless of the request tail (random bytes,
// percent-encoded escapes, dot segments, Windows drive prefixes,
// mixed slashes, embedded NULs) the resulting path MUST stay
// inside the configured `base` directory.
#[cfg(test)]
mod prop_tests {
    use super::{pre_process, sanitize_path};
    use hyper::Request;
    use proptest::prelude::*;
    use std::path::{Component, PathBuf};

    fn assert_under_base(base: &std::path::Path, full: &std::path::Path) {
        let extra = full
            .strip_prefix(base)
            .expect("sanitized path must extend base");
        for comp in extra.components() {
            match comp {
                Component::Normal(_) | Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    panic!("pre_process leaked traversal component {comp:?} in {full:?}");
                }
            }
        }
    }

    fn build_markdown_request(uri_path: &str) -> Request<()> {
        Request::builder()
            .method("GET")
            .uri(uri_path)
            .header("Accept", "text/markdown")
            .body(())
            .unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256, ..ProptestConfig::default()
        })]

        /// Targeted traversal patterns: many `../` segments interleaved
        /// with arbitrary content.
        #[test]
        fn prop_pre_process_resists_dot_dot_floods(
            tail in proptest::collection::vec(
                prop_oneof![
                    Just("../".to_string()),
                    Just("..\\".to_string()),
                    Just("%2e%2e/".to_string()),
                    Just("./".to_string()),
                    "[a-zA-Z0-9_.-]{1,8}".prop_map(String::from),
                ],
                0..32,
            )
        ) {
            let base = PathBuf::from("docker/public").canonicalize().unwrap_or_else(|_| PathBuf::from("docker/public"));
            let tail = tail.concat();
            let uri = format!("/{tail}");
            let req = build_markdown_request(&uri);
            let result = pre_process(&req, &base, &uri, false);

            if let Some(uri_path) = result {
                let sanitized = sanitize_path(&base, &uri_path).expect("returned URI must re-sanitize");
                assert_under_base(&base, &sanitized);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Request;

    fn markdown_request(uri_path: &str) -> Request<()> {
        Request::builder()
            .method("GET")
            .uri(uri_path)
            .header("Accept", "text/markdown")
            .body(())
            .unwrap()
    }

    #[test]
    fn test_no_accept_header() {
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .body(crate::body::empty())
            .unwrap();

        // Without Accept header, should return None (no markdown variant)
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&req, base_path, "/test", false);

        assert!(result.is_none());
    }

    #[test]
    fn test_accepts_html_only() {
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .header("Accept", "text/html")
            .body(crate::body::empty())
            .unwrap();

        // With Accept: text/html, should return None (no markdown variant)
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&req, base_path, "/test", false);

        assert!(result.is_none());
    }

    #[test]
    fn test_accepts_markdown_no_file() {
        let req = markdown_request("/test");

        // With Accept: text/markdown but no file, should return None
        let base_path = std::path::Path::new("/tmp");
        let result = pre_process(&req, base_path, "/test", false);

        assert!(result.is_none());
    }

    #[test]
    fn test_traversal_is_sanitized() {
        let base = std::path::Path::new("tests/fixtures/markdown")
            .canonicalize()
            .unwrap();
        let req = markdown_request("/../README.md");

        // The traversal segment must be stripped, landing inside the base dir
        // where README.md does not exist as a markdown variant.
        let result = pre_process(&req, &base, "/../README.md", false);

        assert!(result.is_none());
    }

    #[test]
    fn test_percent_encoded_traversal_is_sanitized() {
        let base = std::path::Path::new("tests/fixtures/markdown")
            .canonicalize()
            .unwrap();
        let req = markdown_request("/%2e%2e/%2e%2e/README.md");

        let result = pre_process(&req, &base, "/%2e%2e/%2e%2e/README.md", false);

        assert!(result.is_none());
    }

    #[test]
    fn test_hidden_file_rejected() {
        let base = std::path::Path::new("tests/fixtures/public")
            .canonicalize()
            .unwrap();
        let req = markdown_request("/.dotfile");

        // Hidden files are rejected unless include_hidden is enabled.
        let result = pre_process(&req, &base, "/.dotfile", false);
        assert!(result.is_none());

        // When explicitly enabled, the lookup is allowed (the fixture has no
        // markdown variant, so it still returns None, but not because of the
        // hidden check).
        let result = pre_process(&req, &base, "/.dotfile", true);
        assert!(result.is_none());
    }

    #[test]
    fn test_symlink_path_defers_to_static_handler() {
        // The markdown pre-processor doesn't enforce the symlink policy itself;
        // it returns None here (no markdown variant behind the symlink) and the
        // static-files handler applies containment/symlink checks to whatever
        // URI ultimately gets served.
        let base = std::path::Path::new("tests/fixtures/public")
            .canonicalize()
            .unwrap();
        let req = markdown_request("/symlink");

        let result = pre_process(&req, &base, "/symlink", false);
        assert!(result.is_none());
    }

    #[test]
    fn test_existing_markdown_variant_returns_uri() {
        let base = std::path::Path::new("tests/fixtures/markdown")
            .canonicalize()
            .unwrap();
        let req = markdown_request("/article");

        let result = pre_process(&req, &base, "/article", false);

        assert_eq!(result, Some("/article.html.md".to_string()));
    }

    #[test]
    fn test_directory_index_markdown_variant() {
        let base = std::path::Path::new("tests/fixtures/markdown")
            .canonicalize()
            .unwrap();
        let req = markdown_request("/");

        let result = pre_process(&req, &base, "/", false);

        assert_eq!(result, Some("/index.html.md".to_string()));
    }

    #[test]
    fn test_percent_in_filename_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        std::fs::write(base.join("a%2fb.html.md"), "# percent").unwrap();

        // Request decodes to the on-disk file name `a%2fb`.
        let req = markdown_request("/a%252fb");
        let result = pre_process(&req, &base, "/a%252fb", false).unwrap();

        // The returned URI must escape `%` so the static handler's re-decode
        // resolves back to the on-disk file name, not to `a/b.html.md`.
        assert_eq!(result, "/a%252fb.html.md");
        let resolved = sanitize_path(&base, &result).unwrap();
        assert_eq!(resolved, base.join("a%2fb.html.md"));
    }
}
