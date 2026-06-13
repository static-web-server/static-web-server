#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use hyper::Request;
    use std::net::SocketAddr;

    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    async fn content_type(toml: &str, uri: &str) -> String {
        let opts = fixture_settings(toml);
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = uri.parse().unwrap();

        let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.status(), 200);
        res.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[tokio::test]
    async fn default_charset_is_applied() {
        assert_eq!(
            content_type("toml/text_charset_default.toml", "http://localhost/doc.md").await,
            "text/markdown; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn disabled_leaves_content_type_bare() {
        assert_eq!(
            content_type("toml/text_charset_disabled.toml", "http://localhost/doc.md").await,
            "text/markdown"
        );
    }

    #[tokio::test]
    async fn advanced_headers_override_wins() {
        // [[advanced.headers]] runs after text_charset and is meant to win.
        assert_eq!(
            content_type(
                "toml/text_charset_override.toml",
                "http://localhost/article.html",
            )
            .await,
            "text/html; charset=ascii"
        );
    }
}
