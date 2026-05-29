#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for weak `ETag` (`--etag`) support.
//!
//! Exercises RFC 7232 semantics on the public file handler:
//!
//! * `ETag` is emitted on 200/206 responses when enabled.
//! * `If-None-Match` (specific and `*`) yields 304 with validators echoed.
//! * `If-Match` with a weak ETag yields 412 (strong comparison).
//! * `If-Match: *` passes when a representation exists.
//! * `If-Range` with a weak ETag falls back to a full 200.
//! * `--etag false` disables the header entirely.
//! * `ETag` and `Cache-Control` coexist on the full request pipeline.

#[cfg(test)]
mod tests {
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use hyper::Request;
    use std::fs;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    #[cfg(feature = "directory-listing")]
    use static_web_server::directory_listing::DirListFmt;
    use static_web_server::static_files::{self, HandleOpts};
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    fn root_dir() -> PathBuf {
        PathBuf::from("tests/fixtures/public/")
    }

    /// Build `HandleOpts` with default flags. `etag` controls the feature
    /// under test; everything else mirrors the project's existing fixture.
    fn opts<'a>(
        method: &'a Method,
        headers: &'a HeaderMap,
        base: &'a PathBuf,
        uri: &'a str,
        etag: bool,
    ) -> HandleOpts<'a> {
        HandleOpts {
            method,
            headers,
            base_path: base,
            uri_path: uri,
            uri_query: None,
            #[cfg(feature = "mem-cache")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            etag,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &["index.htm"],
        }
    }

    async fn fetch_etag(uri: &str) -> http::HeaderValue {
        let headers = HeaderMap::new();
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, uri, true))
            .await
            .expect("handler should succeed");
        let res = result.resp;
        assert_eq!(res.status(), StatusCode::OK);
        res.headers()
            .get(http::header::ETAG)
            .cloned()
            .expect("ETag header should be present")
    }

    #[tokio::test]
    async fn etag_header_present_on_200() {
        let etag = fetch_etag("index.htm").await;
        let v = etag.to_str().unwrap();
        assert!(v.starts_with("W/\""), "expected weak ETag, got `{v}`");
        assert!(v.ends_with('"'), "expected trailing quote, got `{v}`");
        assert!(v[3..v.len() - 1].contains('-'), "missing `-` in `{v}`");
    }

    #[tokio::test]
    async fn etag_disabled_omits_header() {
        let headers = HeaderMap::new();
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, "index.htm", false))
            .await
            .expect("handler should succeed");
        let res = result.resp;
        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            res.headers().get(http::header::ETAG).is_none(),
            "ETag should be absent when disabled"
        );
    }

    #[tokio::test]
    async fn if_none_match_returns_304_with_etag_and_last_modified() {
        let etag = fetch_etag("index.htm").await;
        let mut headers = HeaderMap::new();
        headers.insert(http::header::IF_NONE_MATCH, etag.clone());
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, "index.htm", true))
            .await
            .expect("handler should succeed");
        let res = result.resp;
        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
        // RFC 7232 §4.1: 304 must echo the validators.
        assert_eq!(
            res.headers().get(http::header::ETAG),
            Some(&etag),
            "304 must echo the matching ETag"
        );
        assert!(
            res.headers().get(http::header::LAST_MODIFIED).is_some(),
            "304 should echo `Last-Modified`"
        );
    }

    #[tokio::test]
    async fn if_none_match_wildcard_returns_304() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::IF_NONE_MATCH,
            http::HeaderValue::from_static("*"),
        );
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, "index.htm", true))
            .await
            .expect("handler should succeed");
        assert_eq!(result.resp.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn if_none_match_mismatch_returns_200() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::IF_NONE_MATCH,
            http::HeaderValue::from_static("W/\"deadbeef-1\""),
        );
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, "index.htm", true))
            .await
            .expect("handler should succeed");
        assert_eq!(result.resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn if_match_with_weak_etag_returns_412() {
        // RFC 7232 §3.1: `If-Match` uses strong comparison; a weak ETag
        // must never satisfy it, even when it matches the resource.
        let etag = fetch_etag("index.htm").await;
        let mut headers = HeaderMap::new();
        headers.insert(http::header::IF_MATCH, etag);
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, "index.htm", true))
            .await
            .expect("handler should succeed");
        assert_eq!(result.resp.status(), StatusCode::PRECONDITION_FAILED);
    }

    #[tokio::test]
    async fn if_match_wildcard_passes() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::IF_MATCH, http::HeaderValue::from_static("*"));
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, "index.htm", true))
            .await
            .expect("handler should succeed");
        assert_eq!(result.resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn if_range_with_weak_etag_falls_back_to_full_200() {
        // RFC 7233 §3.2: `If-Range` uses strong comparison; with a weak
        // ETag the range condition fails and the full representation is
        // served (200 OK), ignoring the `Range` header.
        let etag = fetch_etag("index.htm").await;
        let mut headers = HeaderMap::new();
        headers.insert(http::header::IF_RANGE, etag);
        headers.insert(
            http::header::RANGE,
            http::HeaderValue::from_static("bytes=0-3"),
        );
        let base = root_dir();
        let result = static_files::handle(&opts(&Method::GET, &headers, &base, "index.htm", true))
            .await
            .expect("handler should succeed");
        assert_eq!(result.resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn etag_changes_when_size_changes() {
        // Two fixture files with different sizes must yield distinct ETags.
        let dir = root_dir();
        let p1 = dir.join("_etag_test_a.txt");
        let p2 = dir.join("_etag_test_b.txt");
        fs::write(&p1, b"hi").unwrap();
        fs::write(&p2, b"hello world").unwrap();
        let e1 = fetch_etag("_etag_test_a.txt").await;
        let e2 = fetch_etag("_etag_test_b.txt").await;
        let _ = fs::remove_file(&p1);
        let _ = fs::remove_file(&p2);
        assert_ne!(e1, e2, "different sizes should yield distinct ETags");
    }

    /// `--etag` and `--cache-control-headers` are orthogonal: the static
    /// files handler emits `ETag`; the post-processing `control_headers`
    /// pipeline emits `Cache-Control`. Both must coexist on the same
    /// response without one suppressing the other.
    #[tokio::test]
    async fn etag_coexists_with_cache_control_pipeline() {
        let settings = fixture_settings("toml/handler.toml");
        let req_handler_opts = fixture_req_handler_opts(settings.general, settings.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = "http://localhost/index.html".parse().unwrap();

        let res = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler must succeed");

        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            res.headers().get(http::header::ETAG).is_some(),
            "ETag must be present on full pipeline"
        );
        assert!(
            res.headers().get(http::header::CACHE_CONTROL).is_some(),
            "Cache-Control must coexist with ETag"
        );
    }
}
