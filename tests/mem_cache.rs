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

    use static_web_server::mem_cache::cache::{
        self, DEFAULT_CAPACITY, DEFAULT_MAX_FILE_SIZE, DEFAULT_TTI, DEFAULT_TTL, MemCacheOpts,
    };
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
            ignore_hidden_files: false,
            disable_symlinks: false,
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

        // let mut handler_opts = RequestHandlerOpts::default();
        // handler_opts.advanced_opts = Some(Advanced {
        //     headers: None,
        //     rewrites: None,
        //     redirects: None,
        //     virtual_hosts: None,
        //     memory_cache: Some(MemoryCache {
        //         capacity: Some(10),
        //         ttl: Some(60),
        //         tti: Some(30),
        //         max_file_size: Some(1024),
        //     }),
        // });

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
            ignore_hidden_files: false,
            disable_symlinks: false,
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
            ignore_hidden_files: false,
            disable_symlinks: false,
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
}
