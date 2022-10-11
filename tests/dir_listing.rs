#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use serde::{Deserialize, Serialize};
    use std::path::{Path, PathBuf};

    use static_web_server::{
        directory_listing::DirListFmt,
        static_files::{self, HandleOpts},
    };

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
                base_path: &root_dir("docker/public/"),
                uri_path: "/assets",
                uri_query: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
            })
            .await
            {
                Ok((res, _)) => {
                    assert_eq!(res.status(), 308);
                    assert_eq!(res.headers()["location"], "/assets/");
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
                base_path: &root_dir("docs/"),
                uri_path: "/content/",
                uri_query: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
            })
            .await
            {
                Ok((mut res, _)) => {
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    let body_str = std::str::from_utf8(&body).unwrap();
                    // directory link should only contain "dir-name/" in a relative way
                    assert_eq!(
                        body_str.contains(r#"href="features/""#),
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
                base_path: &root_dir("docs/"),
                uri_path: "/content",
                uri_query: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: false,
                compression_static: false,
            })
            .await
            {
                Ok((mut res, _)) => {
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    let body_str = std::str::from_utf8(&body).unwrap();
                    // directory link should contain "parent/dir-name/" in a relative way
                    assert_eq!(
                        body_str.contains(r#"href="content/features/""#),
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
                base_path: &root_dir("docs/"),
                uri_path: "/README.md",
                uri_query: None,
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: false,
                compression_static: false,
            })
            .await
            {
                Ok((res, _)) => {
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
                dir_listing: true,
                dir_listing_order: 6,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
            })
            .await
            {
                Ok((mut res, _)) => {
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    let body_str = std::str::from_utf8(&body).unwrap();

                    assert_eq!(
                        body_str.contains(
                            r#"<a href="sp%C3%A9cial%20direct%C3%B6ry/">spécial directöry/</a>"#
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
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Json,
                redirect_trailing_slash: true,
                compression_static: false,
            })
            .await
            {
                Ok((mut res, _)) => {
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/json");

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");
                    let body_str = std::str::from_utf8(&body).unwrap();

                    if method == Method::GET {
                        let entries: Vec<FileEntry> = serde_json::from_str(body_str).unwrap();
                        assert_eq!(entries.len(), 2);

                        let first_entry = entries.first().unwrap();
                        assert_eq!(first_entry.name, "spécial directöry");
                        assert_eq!(first_entry.typed, "directory");
                        assert_eq!(first_entry.mtime.is_empty(), false);
                        assert!(first_entry.size.is_none());

                        let last_entry = entries.last().unwrap();
                        assert_eq!(last_entry.name, "index.html.gz");
                        assert_eq!(last_entry.typed, "file");
                        assert_eq!(last_entry.mtime.is_empty(), false);
                        assert!(last_entry.size.unwrap() > 300);
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
}
