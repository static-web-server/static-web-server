#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(feature = "directory-listing-download")]
#[cfg(test)]
mod tests {
    use async_compression::tokio::bufread::GzipDecoder;
    use async_tar::Archive;
    use futures_util::StreamExt;
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use http_body_util::BodyExt;
    use std::{
        collections::HashSet,
        path::{Path, PathBuf},
        pin::Pin,
    };
    use tokio::{fs, io::AsyncReadExt};
    use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

    use static_web_server::{
        directory_listing::DirListFmt,
        directory_listing::download::DirDownloadOpts,
        static_files::{self, HandleOpts},
    };

    use static_web_server::directory_listing::download::{DOWNLOAD_PARAM_KEY, DirDownloadFmt};
    use static_web_server::is_path_within_base;

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

    const OUTSIDE_ROOT_MARKER: &str = "outside-root-marker\n";
    const OUTSIDE_DIR_MARKER: &str = "outside-dir-marker\n";
    const INSIDE_OK_MARKER: &str = "inside-ok\n";

    fn root_dir<P: AsRef<Path>>(dir: P) -> PathBuf
    where
        PathBuf: From<P>,
    {
        PathBuf::from(dir)
    }

    async fn inspect_tarball_content(
        prefix: PathBuf,
        body: &[u8],
        validate: bool,
    ) -> HashSet<PathBuf> {
        let reader = Archive::new(GzipDecoder::new(body).compat());

        let mut content = HashSet::new();
        // adapted from async_tar::Archive::unpack
        let mut entries = reader.entries().unwrap();
        let mut pinned = Pin::new(&mut entries);
        while let Some(entry) = pinned.next().await {
            let file = entry.unwrap();
            let path: PathBuf = file.header().path().unwrap().to_path_buf().into();

            // validate content
            if validate
                && (file.header().entry_type() == async_tar::EntryType::Link
                    || file.header().entry_type() == async_tar::EntryType::Regular
                    || file.header().entry_type() == async_tar::EntryType::Symlink)
            {
                let on_disk_path = prefix.join(&path);
                // in case of symlink, skip dir
                let meta = std::fs::metadata(&on_disk_path).unwrap();
                if !meta.is_dir() {
                    let on_disk = std::fs::read(&on_disk_path).unwrap();
                    let mut compressed = Vec::new();
                    file.compat().read_to_end(&mut compressed).await.unwrap();
                    assert_eq!(on_disk, compressed);
                }
            }

            content.insert(path);
        }
        content
    }

    /// Inspect archive member paths and concatenate regular-file contents.
    async fn inspect_tarball_paths_and_contents(body: &[u8]) -> (HashSet<PathBuf>, Vec<u8>) {
        let reader = Archive::new(GzipDecoder::new(body).compat());
        let mut paths = HashSet::new();
        let mut contents = Vec::new();
        let mut entries = reader.entries().unwrap();
        let mut pinned = Pin::new(&mut entries);
        while let Some(entry) = pinned.next().await {
            let file = entry.unwrap();
            let path: PathBuf = file.header().path().unwrap().to_path_buf().into();
            paths.insert(path);
            if file.header().entry_type() == async_tar::EntryType::Regular {
                let mut buf = Vec::new();
                file.compat().read_to_end(&mut buf).await.unwrap();
                contents.extend_from_slice(&buf);
            }
        }
        (paths, contents)
    }

    async fn get_dir_content(
        path: PathBuf,
        src_path: PathBuf,
        opts: DirDownloadOpts<'_>,
    ) -> HashSet<PathBuf> {
        let mut content = HashSet::new();
        let mut stack = vec![(src_path.to_path_buf(), true, false)];

        while let Some((src, is_dir, is_symlink)) = stack.pop() {
            if !is_path_within_base(&src, opts.base_path) {
                continue;
            }

            let dest = path.join(src.strip_prefix(&src_path).unwrap());

            // In case of a symlink pointing to a directory, is_dir is false, but src.is_dir() will return true
            if is_dir || (is_symlink && opts.follow_symlinks && src.is_dir()) {
                let mut entries = fs::read_dir(&src).await.unwrap();
                while let Some(entry) = entries.next_entry().await.unwrap() {
                    // Check and ignore the current hidden file/directory (dotfile) if feature enabled
                    let name = entry.file_name();
                    if !opts.include_hidden
                        && name.as_encoded_bytes().first().is_some_and(|c| *c == b'.')
                    {
                        continue;
                    }

                    let file_type = entry.file_type().await.unwrap();
                    stack.push((entry.path(), file_type.is_dir(), file_type.is_symlink()));
                }
                if dest != Path::new("") {
                    content.insert(dest);
                }
            } else {
                content.insert(dest);
            }
        }

        content
    }

    /// Build the GHSA-3p6p-6r38-h33v PoC layout under a temp directory.
    ///
    /// Layout:
    /// ```text
    /// tmp/
    ///   webroot/
    ///     safe.txt
    ///     escape_file -> ../secret.txt
    ///     escape_dir  -> ../outside_dir
    ///     ok_link     -> ./safe.txt
    ///   secret.txt
    ///   outside_dir/leaked.txt
    /// ```
    #[cfg(unix)]
    fn create_escape_fixture() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let webroot = tmp.path().join("webroot");
        let outside_dir = tmp.path().join("outside_dir");
        std::fs::create_dir_all(&webroot).expect("webroot");
        std::fs::create_dir_all(&outside_dir).expect("outside_dir");
        std::fs::write(webroot.join("safe.txt"), INSIDE_OK_MARKER).expect("safe.txt");
        std::fs::write(tmp.path().join("secret.txt"), OUTSIDE_ROOT_MARKER).expect("secret.txt");
        std::fs::write(outside_dir.join("leaked.txt"), OUTSIDE_DIR_MARKER).expect("leaked.txt");

        std::os::unix::fs::symlink("../secret.txt", webroot.join("escape_file"))
            .expect("escape_file symlink");
        std::os::unix::fs::symlink("../outside_dir", webroot.join("escape_dir"))
            .expect("escape_dir symlink");
        std::os::unix::fs::symlink("./safe.txt", webroot.join("ok_link")).expect("ok_link symlink");

        (tmp, webroot)
    }

    async fn download_directory(
        webroot: &PathBuf,
        follow_symlinks: bool,
    ) -> (StatusCode, bytes::Bytes) {
        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &HeaderMap::new(),
            base_path: webroot,
            uri_path: "/",
            uri_query: Some(DOWNLOAD_PARAM_KEY),
            #[cfg(feature = "mem-cache")]
            memory_cache: None,
            dir_listing: true,
            dir_listing_order: 1,
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: false,
            etag: true,
            include_hidden: true,
            follow_symlinks,
            index_files: &[],
            dir_listing_download: &[DirDownloadFmt::Targz],
        })
        .await
        .expect("download handle");

        let res = result.resp;
        let status = res.status();
        let body = res
            .into_body()
            .collect()
            .await
            .expect("unexpected bytes error during `body` conversion")
            .to_bytes();
        (status, body)
    }

    fn assert_no_outside_leak(paths: &HashSet<PathBuf>, contents: &[u8]) {
        let contents_str = String::from_utf8_lossy(contents);
        assert!(
            !contents_str.contains(OUTSIDE_ROOT_MARKER.trim_end()),
            "archive must not contain outside-root-marker"
        );
        assert!(
            !contents_str.contains(OUTSIDE_DIR_MARKER.trim_end()),
            "archive must not contain outside-dir-marker"
        );
        assert!(
            !paths.iter().any(|p| {
                let s = p.to_string_lossy();
                s.contains("leaked.txt") || s.contains("secret.txt")
            }),
            "archive paths must not include leaked/secret member names: {paths:?}"
        );
    }

    #[tokio::test]
    async fn dir_listing_download_targz() {
        let base_path = root_dir("tests/fixtures/public");
        let follow_symlinks = true;
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &base_path,
                uri_path: "/",
                uri_query: Some(DOWNLOAD_PARAM_KEY),
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks,
                index_files: &[],
                dir_listing_download: &[DirDownloadFmt::Targz],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/gzip");
                    assert!(
                        res.headers()["content-disposition"]
                            .to_str()
                            .unwrap()
                            .starts_with("attachment")
                    );

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();

                    if method == Method::GET {
                        let mut prefix = base_path.clone();
                        prefix.pop();
                        let left = inspect_tarball_content(prefix, &body, true).await;
                        let right = get_dir_content(
                            PathBuf::from(base_path.file_name().unwrap()),
                            base_path.clone(),
                            DirDownloadOpts {
                                method: &method,
                                base_path: &base_path,
                                follow_symlinks,
                                include_hidden: true,
                            },
                        )
                        .await;

                        if left != right {
                            eprintln!("left - right {:?}", (left.difference(&right)));
                            eprintln!("right - left {:?}", (right.difference(&left)));
                        }

                        assert_eq!(left, right);

                        // Outside-root symlink fixture (readme.md → ../../../README.md)
                        // must not leak repo README content into the archive.
                        let (_, contents) = inspect_tarball_paths_and_contents(&body).await;
                        let contents_str = String::from_utf8_lossy(&contents);
                        assert!(
                            !contents_str.contains("<h1 align=\"center\">Static Web Server</h1>"),
                            "archive must not pack outside-root README.md via readme.md symlink"
                        );
                    } else {
                        assert!(body.is_empty());
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
    async fn dir_listing_download_targz_no_hidden() {
        let base_path = root_dir("tests/fixtures/public");
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &base_path,
                uri_path: "/",
                uri_query: Some(DOWNLOAD_PARAM_KEY),
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
                    assert_eq!(res.headers()["content-type"], "application/gzip");
                    assert!(
                        res.headers()["content-disposition"]
                            .to_str()
                            .unwrap()
                            .starts_with("attachment")
                    );

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();

                    if method == Method::GET {
                        let mut prefix = base_path.clone();
                        prefix.pop();
                        assert!(
                            !inspect_tarball_content(prefix, &body, false)
                                .await
                                .iter()
                                .any(|path| path.file_name().unwrap() == ".dotfile")
                        );
                    } else {
                        assert!(body.is_empty());
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
    async fn dir_listing_download_targz_no_symlinks() {
        let base_path = root_dir("tests/fixtures/public");
        let follow_symlinks = false;
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &base_path,
                uri_path: "/",
                uri_query: Some(DOWNLOAD_PARAM_KEY),
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: true,
                follow_symlinks,
                index_files: &[],
                dir_listing_download: &[DirDownloadFmt::Targz],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/gzip");
                    assert!(
                        res.headers()["content-disposition"]
                            .to_str()
                            .unwrap()
                            .starts_with("attachment")
                    );

                    let body = res
                        .into_body()
                        .collect()
                        .await
                        .expect("unexpected bytes error during `body` conversion")
                        .to_bytes();

                    if method == Method::GET {
                        let mut prefix = base_path.clone();
                        prefix.pop();
                        let left = inspect_tarball_content(prefix, &body, false).await;
                        let right = get_dir_content(
                            PathBuf::from(base_path.file_name().unwrap()),
                            base_path.clone(),
                            DirDownloadOpts {
                                method: &method,
                                base_path: &base_path,
                                follow_symlinks,
                                include_hidden: true,
                            },
                        )
                        .await;

                        if left != right {
                            eprintln!("left - right {:?}", (left.difference(&right)));
                            eprintln!("right - left {:?}", (right.difference(&left)));
                        }

                        assert_eq!(left, right);
                    } else {
                        assert!(body.is_empty());
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
    async fn dir_listing_download_when_disabled() {
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir("tests/fixtures/public"),
                uri_path: "/",
                uri_query: Some(DOWNLOAD_PARAM_KEY),
                #[cfg(feature = "mem-cache")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                etag: true,
                include_hidden: true,
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
                    assert!(
                        !res.headers()
                            .iter()
                            .any(|(k, _v)| *k == "content-disposition")
                    );
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }

    // --- GHSA-3p6p-6r38-h33v regression: outside-base symlink escapes ---

    #[cfg(unix)]
    #[tokio::test]
    async fn download_skips_outside_symlink_dir_follow_on() {
        // Given: webroot with escape_dir → ../outside_dir and follow_symlinks=true
        let (_tmp, webroot) = create_escape_fixture();

        // When: directory download is requested
        let (status, body) = download_directory(&webroot, true).await;

        // Then: 200 OK and outside-dir content/path must be absent
        assert_eq!(status, StatusCode::OK);
        let (paths, contents) = inspect_tarball_paths_and_contents(&body).await;
        assert_no_outside_leak(&paths, &contents);
        assert!(
            !paths
                .iter()
                .any(|p| p.to_string_lossy().contains("escape_dir")),
            "followed outside dir symlink must not appear as archive members: {paths:?}"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn download_skips_outside_symlink_file_follow_on() {
        // Given: webroot with escape_file → ../secret.txt and follow_symlinks=true
        let (_tmp, webroot) = create_escape_fixture();

        // When: directory download is requested
        let (status, body) = download_directory(&webroot, true).await;

        // Then: 200 OK and outside-root secret must be absent
        assert_eq!(status, StatusCode::OK);
        let (paths, contents) = inspect_tarball_paths_and_contents(&body).await;
        assert_no_outside_leak(&paths, &contents);
        assert!(
            !paths.iter().any(|p| {
                p.file_name()
                    .is_some_and(|n| n == "escape_file" || n == "secret.txt")
            }),
            "outside file symlink must not be packed: {paths:?}"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn download_skips_outside_symlinks_follow_off() {
        // Given: webroot with outside-pointing symlinks and follow_symlinks=false
        let (_tmp, webroot) = create_escape_fixture();

        // When: directory download is requested
        let (status, body) = download_directory(&webroot, false).await;

        // Then: 200 OK and outside markers/paths remain absent
        assert_eq!(status, StatusCode::OK);
        let (paths, contents) = inspect_tarball_paths_and_contents(&body).await;
        assert_no_outside_leak(&paths, &contents);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn download_keeps_inside_symlink_when_follow_on() {
        // Given: webroot with ok_link → ./safe.txt and follow_symlinks=true
        let (_tmp, webroot) = create_escape_fixture();

        // When: directory download is requested
        let (status, body) = download_directory(&webroot, true).await;

        // Then: inside content is present (via safe.txt and/or followed ok_link)
        assert_eq!(status, StatusCode::OK);
        let (paths, contents) = inspect_tarball_paths_and_contents(&body).await;
        let contents_str = String::from_utf8_lossy(&contents);
        assert!(
            contents_str.contains(INSIDE_OK_MARKER.trim_end()),
            "archive must keep inside-ok content when following in-root symlink"
        );
        assert!(
            paths.iter().any(|p| {
                p.file_name()
                    .is_some_and(|n| n == "safe.txt" || n == "ok_link")
            }),
            "archive must include safe.txt or ok_link: {paths:?}"
        );
        assert_no_outside_leak(&paths, &contents);
    }
}
