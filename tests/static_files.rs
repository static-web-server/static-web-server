#![deny(warnings)]

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use headers::HeaderMap;
    use http::Method;
    use std::fs;
    use std::path::PathBuf;

    extern crate static_web_server;
    use static_web_server::static_files;

    fn root_dir() -> PathBuf {
        PathBuf::from("docker/public/")
    }

    #[tokio::test]
    async fn handle_file() {
        let mut res = static_files::handle(
            &Method::GET,
            &HeaderMap::new(),
            root_dir(),
            "index.html",
            false,
        )
        .await
        .expect("unexpected response error on `handle` function");

        let buf = fs::read(root_dir().join("index.html"))
            .expect("unexpected error during index.html reading");
        let buf = Bytes::from(buf);

        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-length"], buf.len().to_string());
        assert_eq!(res.headers()["accept-ranges"], "bytes");

        let ctype = &res.headers()["content-type"];

        assert!(
            ctype == "text/html",
            "content-type is not html: {:?}",
            ctype,
        );

        let body = hyper::body::to_bytes(res.body_mut())
            .await
            .expect("unexpected bytes error during `body` convertion");

        assert_eq!(body, buf);
    }
}
