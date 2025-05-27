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
    use std::{
        collections::HashSet,
        path::{Path, PathBuf},
        pin::Pin,
    };
    use tokio::{fs, io::AsyncReadExt};
    use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

    use static_web_server::{
        directory_listing::DirListFmt,
        directory_listing_download::DirDownloadOpts,
        static_files::{self, HandleOpts},
    };

    use static_web_server::directory_listing_download::{DirDownloadFmt, DOWNLOAD_PARAM_KEY};

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

    async fn get_and_validate_tarball_content(prefix: PathBuf, body: &[u8]) -> HashSet<PathBuf> {
        let reader = Archive::new(GzipDecoder::new(body).compat());

        let mut content = HashSet::new();
        // adapted from async_tar::Archive::unpack
        let mut entries = reader.entries().unwrap();
        let mut pinned = Pin::new(&mut entries);
        while let Some(entry) = pinned.next().await {
            let file = entry.unwrap();
            let path: PathBuf = file.header().path().unwrap().to_path_buf().into();

            // validate content
            if file.header().entry_type() == async_tar::EntryType::Link
                || file.header().entry_type() == async_tar::EntryType::Regular
                || file.header().entry_type() == async_tar::EntryType::Symlink
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

    async fn get_dir_content(
        path: PathBuf,
        src_path: PathBuf,
        opts: DirDownloadOpts<'_>,
    ) -> HashSet<PathBuf> {
        let mut content = HashSet::new();
        let mut stack = vec![(src_path.to_path_buf(), true, false)];

        while let Some((src, is_dir, is_symlink)) = stack.pop() {
            let dest = path.join(src.strip_prefix(&src_path).unwrap());

            // In case of a symlink pointing to a directory, is_dir is false, but src.is_dir() will return true
            if is_dir || (is_symlink && !opts.disable_symlinks && src.is_dir()) {
                let mut entries = fs::read_dir(&src).await.unwrap();
                while let Some(entry) = entries.next_entry().await.unwrap() {
                    // Check and ignore the current hidden file/directory (dotfile) if feature enabled
                    let name = entry.file_name();
                    if opts.ignore_hidden_files
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

    #[tokio::test]
    async fn dir_listing_download_targz() {
        let base_path = root_dir("tests/fixtures/public");
        let disable_symlinks = false;
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &base_path,
                uri_path: "/",
                uri_query: Some(DOWNLOAD_PARAM_KEY),
                #[cfg(feature = "experimental")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks,
                index_files: &[],
                dir_listing_download: &vec![DirDownloadFmt::Targz],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/gzip");
                    assert!(res.headers()["content-disposition"]
                        .to_str()
                        .unwrap()
                        .starts_with("attachment"));

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");

                    if method == Method::GET {
                        let mut prefix = base_path.clone();
                        prefix.pop();
                        let left = get_and_validate_tarball_content(prefix, &body).await;
                        let right = get_dir_content(
                            PathBuf::from(base_path.file_name().unwrap()),
                            base_path.clone(),
                            DirDownloadOpts {
                                method: &method,
                                disable_symlinks,
                                ignore_hidden_files: false,
                            },
                        ).await;

                        if left != right {
                            eprintln!("left - right {:?}", (left.difference(&right)));
                            eprintln!("right - left {:?}", (right.difference(&left)));
                        }

                        assert_eq!(left, right);
                    } else {
                        assert!(body.len() == 0);
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
                #[cfg(feature = "experimental")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: true,
                disable_symlinks: false,
                index_files: &[],
                dir_listing_download: &vec![DirDownloadFmt::Targz],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/gzip");
                    assert!(res.headers()["content-disposition"]
                        .to_str()
                        .unwrap()
                        .starts_with("attachment"));

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");

                    if method == Method::GET {
                        let mut prefix = base_path.clone();
                        prefix.pop();
                        assert!(get_and_validate_tarball_content(prefix, &body)
                            .await
                            .iter()
                            .find(|path| path.file_name().unwrap() == ".dotfile")
                            .is_none());
                    } else {
                        assert!(body.len() == 0);
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
        let disable_symlinks = true;
        for method in METHODS {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &base_path,
                uri_path: "/",
                uri_query: Some(DOWNLOAD_PARAM_KEY),
                #[cfg(feature = "experimental")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks,
                index_files: &[],
                dir_listing_download: &vec![DirDownloadFmt::Targz],
            })
            .await
            {
                Ok(result) => {
                    let mut res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "application/gzip");
                    assert!(res.headers()["content-disposition"]
                        .to_str()
                        .unwrap()
                        .starts_with("attachment"));

                    let body = hyper::body::to_bytes(res.body_mut())
                        .await
                        .expect("unexpected bytes error during `body` conversion");

                    if method == Method::GET {
                        let mut prefix = base_path.clone();
                        prefix.pop();
                        let left = get_and_validate_tarball_content(prefix, &body).await;
                        let right = get_dir_content(
                            PathBuf::from(base_path.file_name().unwrap()),
                            base_path.clone(),
                            DirDownloadOpts {
                                method: &method,
                                disable_symlinks,
                                ignore_hidden_files: false,
                            },
                        ).await;

                        if left != right {
                            eprintln!("left - right {:?}", (left.difference(&right)));
                            eprintln!("right - left {:?}", (right.difference(&left)));
                        }

                        assert_eq!(left, right);
                    } else {
                        assert!(body.len() == 0);
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
                #[cfg(feature = "experimental")]
                memory_cache: None,
                dir_listing: true,
                dir_listing_order: 1,
                dir_listing_format: &DirListFmt::Html,
                redirect_trailing_slash: true,
                compression_static: false,
                ignore_hidden_files: false,
                disable_symlinks: false,
                index_files: &[],
                dir_listing_download: &vec![],
            })
            .await
            {
                Ok(result) => {
                    let res = result.resp;
                    assert_eq!(res.status(), 200);
                    assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");
                    assert!(res
                        .headers()
                        .iter()
                        .find(|(k, _v)| *k == "content-disposition")
                        .is_none());
                }
                Err(status) => {
                    assert!(method != Method::GET && method != Method::HEAD);
                    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }
    }
}
