#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use headers::HeaderMap;
    use http::{Method, StatusCode};
    use std::path::PathBuf;

    use static_web_server::static_files::{self, HandleOpts};

    fn root_dir() -> PathBuf {
        PathBuf::from("docker/public/")
    }

    #[tokio::test]
    async fn dir_listing_redirect_permanent_uri() {
        let methods = [
            Method::CONNECT,
            Method::DELETE,
            Method::GET,
            Method::HEAD,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ];
        for method in methods {
            match static_files::handle(&HandleOpts {
                method: &method,
                headers: &HeaderMap::new(),
                base_path: &root_dir(),
                uri_path: "/assets",
                uri_query: None,
                dir_listing: true,
                dir_listing_order: 6,
                redirect_trailing_slash: true,
            })
            .await
            {
                Ok(res) => {
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
}
