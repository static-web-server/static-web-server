#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::Method;
    use std::path::PathBuf;

    use static_web_server::{
        directory_listing::DirListFmt,
        static_files::{self, HandleOpts},
    };

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

        let (mut resp, _) = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &public_dir(),
            uri_path: "index.html",
            uri_query: None,
            dir_listing: false,
            dir_listing_order: 6,
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: true,
            ignore_hidden_files: false,
        })
        .await
        .expect("unexpected error response on `handle` function");

        let index_gz_buf =
            std::fs::read(&index_gz_path).expect("unexpected error when reading index.html.gz");
        let index_gz_buf = Bytes::from(index_gz_buf);

        std::fs::remove_file(index_gz_path_public).unwrap();

        let headers = resp.headers();

        assert_eq!(resp.status(), 200);
        assert!(!headers.contains_key("content-length"));
        assert_eq!(headers["content-encoding"], "gzip");
        assert_eq!(headers["accept-ranges"], "bytes");
        assert!(!headers["last-modified"].is_empty());
        assert_eq!(
            &headers["content-type"], "text/html",
            "content-type is not html"
        );

        let body = hyper::body::to_bytes(resp.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(
            body, index_gz_buf,
            "body and index_gz_buf are not equal in length"
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

        let (mut resp, _) = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &public_dir().join("assets/"),
            uri_path: "index.html",
            uri_query: None,
            dir_listing: false,
            dir_listing_order: 6,
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: true,
            ignore_hidden_files: false,
        })
        .await
        .expect("unexpected error response on `handle` function");

        let index_buf =
            std::fs::read(&index_path_public).expect("unexpected error when reading index.html");
        let index_buf = Bytes::from(index_buf);

        let headers = resp.headers();

        assert_eq!(resp.status(), 200);
        assert!(headers.contains_key("content-length"));
        assert_eq!(headers["accept-ranges"], "bytes");
        assert!(!headers["last-modified"].is_empty());
        assert_eq!(
            &headers["content-type"], "text/html",
            "content-type is not html"
        );

        let body = hyper::body::to_bytes(resp.body_mut())
            .await
            .expect("unexpected bytes error during `body` conversion");

        assert_eq!(
            body, index_buf,
            "body and index_gz_buf are not equal in length"
        );
    }

    #[tokio::test]
    async fn compression_static_base_path_as_dot() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::ACCEPT_ENCODING,
            "gzip, deflate, br".parse().unwrap(),
        );

        let base_path = PathBuf::from(".");

        let (_resp, _) = static_files::handle(&HandleOpts {
            method: &Method::GET,
            headers: &headers,
            base_path: &base_path,
            uri_path: "/",
            uri_query: None,
            dir_listing: true,
            dir_listing_order: 6,
            dir_listing_format: &DirListFmt::Html,
            redirect_trailing_slash: true,
            compression_static: true,
            ignore_hidden_files: false,
        })
        .await
        .expect("unexpected error response on `handle` function");
    }
}
