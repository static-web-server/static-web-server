#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(feature = "directory-listing")]
#[cfg(test)]
mod tests {
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use http_body_util::BodyExt;
    use serde::{Deserialize, Serialize};
    use std::path::{Path, PathBuf};

    use static_web_server::{
        directory_listing::DirListFmt,
        static_files::{self, HandleOpts},
    };

    #[cfg(feature = "directory-listing-download")]
    use static_web_server::directory_listing::download::DirDownloadFmt;

    const METHODS: [Method; 8] = [
        Method::CONNECT,
        Method::DELETE,
        Method::GET,
        Method::HEAD,
        Method::PATCH,
        Method::POST,
        Method::PUT,
        Method::TRACE,
    ];

    fn root_dir<P: AsRef<Path>>(dir: P) -> PathBuf
    where
        PathBuf: From<P>,
    {
        PathBuf::from(dir)
    }

    #[tokio::test]
    async fn dir_listing_redirect_trailing_slash_dir() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/public"),
                uri_path: "/symlink",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 308);
                    assert_eq!(res.headers()["location"], "/symlink/");
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    async fn dir_listing_redirect_trailing_slash_relative_dir_path() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures"),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();
                    // directory link should only contain "dir-name/" in a relative way
                    assert_eq!(
                        body_str.contains(r#"href="markdown/""#),
                        method == Method::GET
                    );
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    async fn dir_listing_no_redirect_trailing_slash_relative_dir_path() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures"),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: false,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();
                    // directory link should contain "parent/dir-name/" in a relative way
                    assert_eq!(
                        body_str.contains(r#"href="markdown/""#),
                        method == Method::GET
                    );
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    async fn dir_listing_no_redirect_trailing_slash_relative_file_path() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/markdown/"),
                uri_path: "/article.html.md",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: false,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/markdown");
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    async fn dir_listing_links_properly_encoded() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/public/"),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();

                    assert_eq!(
                        body_str.contains(
                            r#"<a href="sp%C3%A9cial-direct%C3%B6ry.net/">spécial-directöry.net/</a>"#
                        ),
                        method == Method::GET
                    );
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    async fn dir_listing_json_format() {
        #[derive(Serialize, Deserialize)]
        struct FileEntry {
            name: String,
            #[serde(rename = "type")]
            typed: String,
            mtime: String,
            size: Option<usize>,
        }

        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/public/"),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Json,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/json");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();

                    if method == Method::GET {
                        let entries: Vec<FileEntry> = serde_json::from_str(body_str).unwrap();
                        assert_eq!(entries.len(), 9);

                        let first_entry = entries.first().unwrap();
                        assert_eq!(first_entry.name, "symlink");
                        assert_eq!(first_entry.typed, "directory");
                        assert!(!first_entry.mtime.is_empty());
                        assert!(first_entry.size.is_none());

                        let last_entry = entries.last().unwrap();
                        assert_eq!(last_entry.name, "404.html");
                        assert_eq!(last_entry.typed, "file");
                        assert!(!last_entry.mtime.is_empty());
                        assert!(last_entry.size.unwrap() > 60);
                    } else {
                        assert!(body_str.is_empty());
                    }
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    async fn dir_listing_json_format_empty() {
        #[derive(Serialize, Deserialize)]
        struct FileEntry {
            name: String,
            #[serde(rename = "type")]
            typed: String,
            mtime: String,
            size: Option<usize>,
        }

        let empty_dir = PathBuf::from("tests/fixtures/empty");
        if empty_dir.exists() {
            std::fs::remove_dir(&empty_dir).unwrap();
        }
        std::fs::create_dir(&empty_dir).unwrap();

        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(&empty_dir),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Json,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/json");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();

                    if method == Method::GET {
                        let entries: Vec<FileEntry> = serde_json::from_str(body_str).unwrap();
                        assert!(entries.is_empty())
                    } else {
                        assert!(body_str.is_empty());
                    }
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    async fn dir_listing_include_hidden() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/public"),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
                index_files: &[],
                #[cfg(feature = "directory-listing-download")]
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();

                    if method == Method::GET {
                        assert!(!body_str.contains(".dotfile"))
                    } else {
                        assert!(body_str.is_empty());
                    }
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    #[cfg(feature = "directory-listing-download")]
    async fn dir_listing_has_download_link_when_enabled() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/public"),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
                index_files: &[],
                dir_listing_download: &[DirDownloadFmt::Targz],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();

                    if method == Method::GET {
                        assert!(body_str.contains("download tar.gz"))
                    } else {
                        assert!(body_str.is_empty());
                    }
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    #[tokio::test]
    #[cfg(feature = "directory-listing-download")]
    async fn dir_listing_has_no_download_link_when_disabled() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/public"),
                uri_path: "/",
                uri_query: None,
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: false,
                follow_symlinks: true,
                index_files: &[],
                dir_listing_download: &[],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();
                    let body_str = std::str::from_utf8(&body).unwrap();

                    if method == Method::GET {
                        assert!(!body_str.contains("download tar.gz"))
                    } else {
                        assert!(body_str.is_empty());
                    }
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    /// SECURITY (XSS): regression test ensuring that filenames containing
    /// HTML metacharacters are escaped, not reflected, in the autoindex
    /// HTML output.
    ///
    /// We materialise a temp directory with a file whose name embeds a
    /// `<script>` payload, render the listing, and assert that the
    /// payload appears only in its escaped form. Disabled on Windows
    /// because NTFS forbids `<` and `>` in filenames.
    #[cfg(unix)]
    #[tokio::test]
    async fn dir_listing_escapes_html_in_filenames() {
        use std::fs;

        let tmp = std::env::temp_dir().join(format!(
            "sws-xss-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&tmp).unwrap();
        // NOTE: filename intentionally avoids `/` (path separator) but
        // contains other HTML metacharacters that must be escaped.
        let evil_name = "<script>alert(1)<_script>.txt";
        let evil = tmp.join(evil_name);
        fs::write(&evil, b"x").unwrap();

        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: &tmp,
            uri_path: "/",
            uri_query: None,
            #[cfg(feature = "mem-cache")]
            memory_cache: None,
            dir_listing: true,
            dir_listing_order: 6,
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: false,
            etag: true,
            include_hidden: true,
            follow_symlinks: true,
            index_files: &[],
            #[cfg(feature = "directory-listing-download")]
            dir_listing_download: &[],
        })
        .await
        .expect("handle should succeed for GET on a readable directory");

        let body = result
            .resp
            .into_body()
            .collect()
            .await
            .expect("body collect")
            .to_bytes();
        let body_str = std::str::from_utf8(&body).unwrap();

        assert!(
            !body_str.contains("<script>alert(1)<_script>"),
            "raw <script> payload leaked into directory listing"
        );
        assert!(
            body_str.contains("&lt;script&gt;alert(1)&lt;_script&gt;"),
            "expected HTML-escaped filename in listing; body was: {body_str}"
        );

        // Cleanup
        let _ = fs::remove_file(&evil);
        let _ = fs::remove_dir(&tmp);
    }
}
