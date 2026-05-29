#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(all(test, feature = "mem-cache"))]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use http_body_util::BodyExt;
    use std::path::PathBuf;

    #[cfg(feature = "directory-listing")]
    use static_web_server::directory_listing::DirListFmt;
    use static_web_server::static_files::{self, HandleOpts};

    use static_web_server::handler::RequestHandlerOpts;
    use static_web_server::mem_cache::cache::{
        self, DEFAULT_CAPACITY, DEFAULT_MAX_FILE_SIZE, DEFAULT_TTI, DEFAULT_TTL, MemCacheOpts,
    };
    use static_web_server::settings::Advanced;
    use static_web_server::settings::file::MemoryCache;

    fn root_dir() -> PathBuf {
        PathBuf::from("tests/fixtures/public/")
    }

    fn handle_opts(memory_cache: Option<&MemCacheOpts>) -> HandleOpts<'static> {
        // Leak references for test convenience (short-lived test processes).
        let mc: Option<&'static MemCacheOpts> = memory_cache.map(|m| {
            let boxed = Box::new(MemCacheOpts::new(m.max_file_size / 1024));
            &*Box::leak(boxed)
        });
        HandleOpts {
            method: &Method::GET,
            headers: Box::leak(Box::new(HeaderMap::new())),
            base_path: Box::leak(Box::new(root_dir())),
            uri_path: "index.htm",
            uri_query: None,
            memory_cache: mc,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: Box::leak(Box::new(DirListFmt::Html)),
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &["index.htm"],
        }
    }

    #[test]
    fn memory_cache_config_defaults() {
        let cfg = MemoryCache {
            capacity: None,
            ttl: None,
            tti: None,
            max_file_size: None,
        };
        assert_eq!(cfg.capacity.unwrap_or(DEFAULT_CAPACITY), 100);
        assert_eq!(cfg.ttl.unwrap_or(DEFAULT_TTL), 1800);
        assert_eq!(cfg.tti.unwrap_or(DEFAULT_TTI), 300);
        assert_eq!(cfg.max_file_size.unwrap_or(DEFAULT_MAX_FILE_SIZE), 8192);
    }

    #[test]
    fn memory_cache_config_custom_values() {
        let cfg = MemoryCache {
            capacity: Some(50),
            ttl: Some(600),
            tti: Some(120),
            max_file_size: Some(4096),
        };
        assert_eq!(cfg.capacity.unwrap(), 50);
        assert_eq!(cfg.ttl.unwrap(), 600);
        assert_eq!(cfg.tti.unwrap(), 120);
        assert_eq!(cfg.max_file_size.unwrap(), 4096);
    }

    #[test]
    fn memory_cache_config_deserializes_from_toml() {
        let toml_str = r#"
            capacity = 200
            ttl = 900
            tti = 60
            max-file-size = 16384
        "#;
        let cfg: MemoryCache = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.capacity, Some(200));
        assert_eq!(cfg.ttl, Some(900));
        assert_eq!(cfg.tti, Some(60));
        assert_eq!(cfg.max_file_size, Some(16384));
    }

    #[test]
    fn memory_cache_config_deserializes_empty_toml() {
        let toml_str = "";
        let cfg: MemoryCache = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.capacity, None);
        assert_eq!(cfg.ttl, None);
        assert_eq!(cfg.tti, None);
        assert_eq!(cfg.max_file_size, None);
    }

    #[test]
    fn memory_cache_config_partial_toml() {
        let toml_str = r#"
            capacity = 50
            max-file-size = 1024
        "#;
        let cfg: MemoryCache = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.capacity, Some(50));
        assert_eq!(cfg.ttl, None);
        assert_eq!(cfg.tti, None);
        assert_eq!(cfg.max_file_size, Some(1024));
    }

    #[test]
    fn mem_cache_opts_file_size_conversion() {
        // 8192 KiB = 8 MiB in bytes
        let opts = MemCacheOpts::new(8192);
        assert_eq!(opts.max_file_size, 8192 * 1024);
    }

    #[test]
    fn cache_init_populates_store() {
        use static_web_server::handler::RequestHandlerOpts;
        use static_web_server::settings::Advanced;

        let memory_cache = Some(MemoryCache {
            capacity: Some(10),
            ttl: Some(60),
            tti: Some(30),
            max_file_size: Some(1024),
        });
        let mut handler_opts = RequestHandlerOpts {
            advanced_opts: Some(Advanced {
                headers: None,
                rewrites: None,
                redirects: None,
                virtual_hosts: None,
                memory_cache,
            }),
            ..Default::default()
        };

        let result = cache::init(&mut handler_opts);
        assert!(result.is_ok());
        assert!(handler_opts.memory_cache.is_some());
        let opts = handler_opts.memory_cache.as_ref().unwrap();
        assert_eq!(opts.max_file_size, 1024 * 1024); // 1024 KiB in bytes
    }

    #[tokio::test]
    async fn static_file_served_with_cache_enabled() {
        let mem_opts = MemCacheOpts::new(DEFAULT_MAX_FILE_SIZE);
        let opts = handle_opts(Some(&mem_opts));

        let result = static_files::handle(&opts).await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert_eq!(resp.resp.status(), StatusCode::OK);

        let body = resp.resp.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn static_file_served_without_cache() {
        let opts = handle_opts(None);
        let result = static_files::handle(&opts).await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert_eq!(resp.resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn cache_miss_then_hit_returns_same_content() {
        let mem_opts = MemCacheOpts::new(DEFAULT_MAX_FILE_SIZE);

        // First request: cache miss, populates cache
        let opts1 = HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: &root_dir(),
            uri_path: "index.htm",
            uri_query: None,
            memory_cache: Some(&mem_opts),
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
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &["index.htm"],
        };

        let result1 = static_files::handle(&opts1).await;
        assert!(result1.is_ok());
        let body1: Bytes = result1
            .unwrap()
            .resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();

        // Second request: should be served from cache (or at least produce same content)
        let opts2 = HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: &root_dir(),
            uri_path: "index.htm",
            uri_query: None,
            memory_cache: Some(&mem_opts),
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
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &["index.htm"],
        };

        let result2 = static_files::handle(&opts2).await;
        assert!(result2.is_ok());
        let body2: Bytes = result2
            .unwrap()
            .resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();

        assert_eq!(body1, body2);
    }

    // Runtime gating via TOML config file.
    //
    // The in-memory cache must only activate when `[advanced.memory-cache]` is
    // present in the configuration file. Otherwise the server must behave as
    // if the feature were not compiled in (no allocation, no global state).

    #[test]
    fn toml_fixture_enables_memory_cache() {
        use static_web_server::settings::file::Settings;
        use std::path::Path;

        let config_path = Path::new("tests/toml/memory_cache.toml");
        let settings = Settings::read(config_path).unwrap();
        let advanced = settings.advanced.expect("`[advanced]` section missing");
        let mc = advanced
            .memory_cache
            .expect("`[advanced.memory-cache]` section missing");

        assert_eq!(mc.capacity, Some(50));
        assert_eq!(mc.ttl, Some(600));
        assert_eq!(mc.tti, Some(120));
        assert_eq!(mc.max_file_size, Some(4));
    }

    #[test]
    fn toml_fixture_without_memory_cache_section_disables_feature() {
        use static_web_server::settings::file::Settings;
        use std::path::Path;

        let config_path = Path::new("tests/fixtures/toml/handler.toml");
        let settings = Settings::read(config_path).unwrap();
        // The `[advanced]` table may exist but `memory-cache` must be absent.
        let mc = settings.advanced.and_then(|a| a.memory_cache);
        assert!(
            mc.is_none(),
            "memory_cache should be None when `[advanced.memory-cache]` is absent"
        );
    }

    // X-Cache response header.
    //
    // Responses served from the in-memory cache must include `X-Cache: HIT`.
    // Responses from disk (cache miss or cache disabled) must NOT include the
    // header.

    #[tokio::test]
    async fn x_cache_hit_header_present_on_cache_hit() {
        let mut handler_opts = RequestHandlerOpts {
            advanced_opts: Some(Advanced {
                memory_cache: Some(MemoryCache {
                    capacity: Some(DEFAULT_CAPACITY),
                    ttl: Some(DEFAULT_TTL),
                    tti: Some(DEFAULT_TTI),
                    max_file_size: Some(DEFAULT_MAX_FILE_SIZE),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let _ = cache::init(&mut handler_opts);

        let mem_opts = MemCacheOpts::new(DEFAULT_MAX_FILE_SIZE);

        // Use a file that no other test in this binary touches, so we
        // guarantee a true cache miss regardless of test execution order
        // (the CACHE_STORE is a process-global OnceLock).
        let make_opts = || HandleOpts {
            method: &Method::GET,
            headers: Box::leak(Box::new(HeaderMap::new())),
            base_path: Box::leak(Box::new(root_dir())),
            uri_path: "50x.html",
            uri_query: None,
            memory_cache: Some(Box::leak(Box::new(MemCacheOpts::new(
                mem_opts.max_file_size / 1024,
            )))),
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: Box::leak(Box::new(DirListFmt::Html)),
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &[],
        };

        // First request — cache miss (file not yet in store).
        let result1 = static_files::handle(&make_opts()).await;
        let resp1 = result1.expect("first request should succeed").resp;
        assert!(
            resp1.headers().get("x-cache").is_none(),
            "X-Cache should not be set on a cache miss (first request)"
        );

        // Consume the body so the streaming pipeline finishes and populates the cache.
        let _body1 = resp1.into_body().collect().await.unwrap().to_bytes();

        // Second request — should be served from cache (hit).
        let result2 = static_files::handle(&make_opts()).await;
        assert!(result2.is_ok());
        let resp2 = result2.unwrap().resp;
        let x_cache = resp2
            .headers()
            .get("x-cache")
            .expect("X-Cache header must be present on a cache hit");
        assert_eq!(x_cache, "HIT");
    }

    #[tokio::test]
    async fn x_cache_header_absent_when_cache_disabled() {
        // With memory_cache = None, no X-Cache header should ever appear.
        let opts = handle_opts(None);
        let result = static_files::handle(&opts).await;
        assert!(result.is_ok());

        let resp = result.unwrap().resp;
        assert!(
            resp.headers().get("x-cache").is_none(),
            "X-Cache should never appear when the in-memory cache is disabled"
        );
    }

    // ETag on the in-memory cache pipeline.
    //
    // The cache must store the weak `ETag` once (alongside the body bytes)
    // and emit it on every hit. `If-None-Match` against the cached value
    // must yield 304 with the validators echoed.

    #[tokio::test]
    async fn cached_response_carries_etag_and_handles_if_none_match() {
        let mut handler_opts = RequestHandlerOpts {
            advanced_opts: Some(Advanced {
                memory_cache: Some(MemoryCache {
                    capacity: Some(DEFAULT_CAPACITY),
                    ttl: Some(DEFAULT_TTL),
                    tti: Some(DEFAULT_TTI),
                    max_file_size: Some(DEFAULT_MAX_FILE_SIZE),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let _ = cache::init(&mut handler_opts);

        // Distinct file from other tests to keep the CACHE_STORE entry isolated.
        let uri: &'static str = "404.html";

        let make_opts = |hdrs: HeaderMap| HandleOpts {
            method: &Method::GET,
            headers: Box::leak(Box::new(hdrs)),
            base_path: Box::leak(Box::new(root_dir())),
            uri_path: uri,
            uri_query: None,
            memory_cache: Some(Box::leak(Box::new(MemCacheOpts::new(
                DEFAULT_MAX_FILE_SIZE / 1024,
            )))),
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: Box::leak(Box::new(DirListFmt::Html)),
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
            redirect_trailing_slash: true,
            compression_static: false,
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &[],
        };

        // 1) Cache miss — disk path emits ETag and populates the cache.
        let resp1 = static_files::handle(&make_opts(HeaderMap::new()))
            .await
            .expect("first request should succeed")
            .resp;
        assert_eq!(resp1.status(), StatusCode::OK);
        let etag1 = resp1
            .headers()
            .get(http::header::ETAG)
            .cloned()
            .expect("ETag must be present on disk-served response");
        // Drain to complete cache population.
        let _ = resp1.into_body().collect().await.unwrap().to_bytes();

        // 2) Cache hit — same ETag is emitted from cache.
        let resp2 = static_files::handle(&make_opts(HeaderMap::new()))
            .await
            .expect("second request should succeed")
            .resp;
        assert_eq!(resp2.status(), StatusCode::OK);
        assert_eq!(
            resp2.headers().get("x-cache").map(|v| v.as_bytes()),
            Some(&b"HIT"[..]),
            "expected cache hit"
        );
        assert_eq!(
            resp2.headers().get(http::header::ETAG),
            Some(&etag1),
            "cache hit must echo the original ETag"
        );

        // 3) If-None-Match against cached entry → 304 with echoed validators.
        let mut h = HeaderMap::new();
        h.insert(http::header::IF_NONE_MATCH, etag1.clone());
        let resp3 = static_files::handle(&make_opts(h))
            .await
            .expect("third request should succeed")
            .resp;
        assert_eq!(resp3.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(
            resp3.headers().get(http::header::ETAG),
            Some(&etag1),
            "304 from cache must echo the ETag (RFC 7232 §4.1)"
        );
    }
}
