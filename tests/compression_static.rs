#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(any(
    feature = "compression",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd"
))]
#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::Method;
    use std::path::PathBuf;

    #[cfg(feature = "directory-listing")]
    use static_web_server::directory_listing::DirListFmt;
    use static_web_server::static_files::{self, HandleOpts};

    fn public_dir() -> PathBuf {
        PathBuf::from("docker/public/")
    }

    #[tokio::test]
    async fn compression_static_file_exists() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        let index_gz_path = PathBuf::from("tests/fixtures/public/index.html.gz");
        let index_gz_path_public = public_dir().join("index.html.gz");
        std::fs::copy(&index_gz_path, &index_gz_path_public)
            .expect("unexpected error copying fixture file");

        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &public_dir(),
            uri_path: "index.html",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            #[cfg(any(
                feature = "compression",
                feature = "compression-deflate",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd"
            ))]
            compression_static: true,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let index_gz_buf =
            std::fs::read(&index_gz_path).expect("unexpected error when reading index.html.gz");
        let index_gz_buf = Bytes::from(index_gz_buf);

        std::fs::remove_file(index_gz_path_public).unwrap();

        let headers = res.headers();

        assert_eq!(res.status(), 200);
        assert!(!headers.contains_key("content-length"));
        assert_eq!(headers["content-encoding"], "gzip");
        assert_eq!(headers["accept-ranges"], "bytes");
        assert!(!headers["last-modified"].is_empty());
        assert_eq!(
            &headers["content-type"], "text/html",
            "content-type is not html"
        );

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(
            body, index_gz_buf,
            "body and index_gz_buf are not equal in length"
        );
    }

    #[tokio::test]
    async fn compression_static_suboptimal_file_exists() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let index_br_path = PathBuf::from("tests/fixtures/public/404.html.br");
        let index_br_path_public = public_dir().join("404.html.br");
        std::fs::copy(&index_br_path, &index_br_path_public)
            .expect("unexpected error copying fixture file");

        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &public_dir(),
            uri_path: "404.html",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            #[cfg(any(
                feature = "compression",
                feature = "compression-deflate",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd"
            ))]
            compression_static: true,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let index_br_buf =
            std::fs::read(&index_br_path).expect("unexpected error when reading index.html.br");
        let index_br_buf = Bytes::from(index_br_buf);

        std::fs::remove_file(index_br_path_public).unwrap();

        let headers = res.headers();

        assert_eq!(res.status(), 200);
        assert!(!headers.contains_key("content-length"));
        assert_eq!(headers["content-encoding"], "br");
        assert_eq!(headers["accept-ranges"], "bytes");
        assert!(!headers["last-modified"].is_empty());
        assert_eq!(
            &headers["content-type"], "text/html",
            "content-type is not html"
        );

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(
            body, index_br_buf,
            "body and index_br_buf are not equal in length"
        );
    }

    #[tokio::test]
    async fn compression_static_file_does_not_exist() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        let index_path_public = public_dir().join("assets/index.html");

        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &public_dir().join("assets/"),
            uri_path: "index.html",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            #[cfg(any(
                feature = "compression",
                feature = "compression-deflate",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd"
            ))]
            compression_static: true,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let index_buf =
            std::fs::read(&index_path_public).expect("unexpected error when reading index.html");
        let index_buf = Bytes::from(index_buf);

        let headers = res.headers();

        assert_eq!(res.status(), 200);
        assert!(headers.contains_key("content-length"));
        assert_eq!(headers["accept-ranges"], "bytes");
        assert!(!headers["last-modified"].is_empty());
        assert_eq!(
            &headers["content-type"], "text/html",
            "content-type is not html"
        );

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(
            body, index_buf,
            "body and index_gz_buf are not equal in length"
        );
    }

    #[cfg(feature = "directory-listing")]
    #[tokio::test]
    async fn compression_static_base_path_as_dot() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        let base_path = PathBuf::from(".");

        static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &base_path,
            uri_path: "/",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            dir_listing: true,
            dir_listing_order: 6,
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: true,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("unexpected error response on `handle` function");
    }

    #[tokio::test]
    async fn compression_static_zstd_file_exists() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );

        let mainjs_zst_path = PathBuf::from("tests/fixtures/public/main.js.zst");
        let mainjs_zst_path_public = public_dir().join("main.js.zst");
        std::fs::copy(&mainjs_zst_path, &mainjs_zst_path_public)
            .expect("unexpected error copying fixture file");

        let result = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &public_dir(),
            uri_path: "main.js",
            uri_query: None,
            #[cfg(feature = "experimental")]
            memory_cache: None,
            #[cfg(feature = "directory-listing")]
            dir_listing: false,
            #[cfg(feature = "directory-listing")]
            dir_listing_order: 6,
            #[cfg(feature = "directory-listing")]
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            #[cfg(any(
                feature = "compression",
                feature = "compression-deflate",
                feature = "compression-gzip",
                feature = "compression-brotli",
                feature = "compression-zstd"
            ))]
            compression_static: true,
            ignore_hidden_files: false,
            disable_symlinks: false,
            index_files: &[],
        })
        .await
        .expect("unexpected error response on `handle` function");
        let mut res = result.resp;

        let mainjs_zst_buf =
            std::fs::read(&mainjs_zst_path).expect("unexpected error when reading index.html.gz");
        let mainjs_zst_buf = Bytes::from(mainjs_zst_buf);

        std::fs::remove_file(mainjs_zst_path_public).unwrap();

        let headers = res.headers();

        assert_eq!(res.status(), 200);
        assert!(!headers.contains_key("content-length"));
        assert_eq!(headers["content-encoding"], "zstd");
        assert_eq!(headers["accept-ranges"], "bytes");
        assert!(!headers["last-modified"].is_empty());
        assert_eq!(
            &headers["content-type"], "text/javascript",
            "content-type is not javascript"
        );

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(
            body, mainjs_zst_buf,
            "body and mainjs_zst_buf are not equal in length"
        );
    }
}
