#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

// Integration tests for the per-request log line of `log_addr`.
//
// Each test captures the JSON events of its own thread through a scoped
// `tracing` subscriber (`#[tokio::test]` runs on one thread), so the tests
// neither install nor depend on the global subscriber.

#[cfg(test)]
mod tests {
    use hyper::{Method, Request};
    use std::io::Write;
    use std::net::{IpAddr, SocketAddr};
    use std::sync::{Arc, Mutex};

    use static_web_server::handler::RequestHandler;
    use static_web_server::settings::cli::General;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl Write for Capture {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn handler(fixture: &str, general: impl FnOnce(General) -> General) -> RequestHandler {
        let opts = fixture_settings(fixture);
        fixture_req_handler(fixture_req_handler_opts(
            general(opts.general),
            opts.advanced,
        ))
    }

    /// Sends `req` through `handler` and returns the response status and the `log_addr` events.
    async fn request_log_lines(
        handler: &RequestHandler,
        mut req: Request<()>,
    ) -> (u16, Vec<serde_json::Value>) {
        let capture = Capture::default();
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(move || writer.clone())
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        let status = handler
            .handle(&mut req, remote_addr)
            .await
            .expect("unexpected handler error")
            .status()
            .as_u16();
        drop(guard);

        let output = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
        let lines = output
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON"))
            .filter(|line| line["target"] == "static_web_server::log_addr")
            .collect();
        (status, lines)
    }

    fn remote_ip() -> String {
        REMOTE_ADDR.parse::<SocketAddr>().unwrap().ip().to_string()
    }

    #[tokio::test]
    async fn request_logging_file_logs_one_line_with_status_200() {
        let handler = handler("toml/handler.toml", |general| General {
            log_remote_address: true,
            ..general
        });

        for method in [Method::GET, Method::HEAD] {
            let uri = "http://localhost/index.html";
            let req = Request::builder()
                .method(method.clone())
                .uri(uri)
                .body(())
                .unwrap();

            let (status, lines) = request_log_lines(&handler, req).await;

            assert_eq!(status, 200);
            assert_eq!(lines.len(), 1, "one log line per request: {lines:?}");
            assert_eq!(lines[0]["level"], "INFO");
            assert_eq!(lines[0]["method"], method.as_str());
            assert_eq!(lines[0]["uri"], uri);
            assert_eq!(lines[0]["status"], 200);
            assert_eq!(lines[0]["remote_addr"], remote_ip());
        }
    }

    #[tokio::test]
    async fn request_logging_missing_file_logs_status_404() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::get("http://localhost/missing.css")
            .body(())
            .unwrap();

        let (status, lines) = request_log_lines(&handler, req).await;

        assert_eq!(status, 404);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0]["status"], 404);
        assert!(
            lines[0].get("remote_addr").is_none(),
            "remote address logging is off by default: {lines:?}"
        );
    }

    #[tokio::test]
    async fn request_logging_disallowed_method_logs_status_405() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::builder()
            .method(Method::PATCH)
            .uri("http://localhost/index.html")
            .body(())
            .unwrap();

        let (status, lines) = request_log_lines(&handler, req).await;

        assert_eq!(status, 405);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0]["method"], "PATCH");
        assert_eq!(lines[0]["status"], 405);
    }

    #[tokio::test]
    async fn request_logging_redirect_logs_status_301() {
        let handler = handler("toml/redirects.toml", |general| General {
            root: "tests/fixtures/public".into(),
            ..general
        });
        let uri = "http://localhost/pages/about.html";
        let req = Request::get(uri).body(()).unwrap();

        let (status, lines) = request_log_lines(&handler, req).await;

        assert_eq!(status, 301);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0]["uri"], uri);
        assert_eq!(lines[0]["status"], 301);
    }

    #[tokio::test]
    async fn request_logging_rewrite_logs_the_uri_sent_by_the_client() {
        let handler = handler("toml/rewrites.toml", |general| general);
        let uri = "http://localhost/some/error-page.html";
        let req = Request::get(uri).body(()).unwrap();

        let (status, lines) = request_log_lines(&handler, req).await;

        assert_eq!(status, 200);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(
            lines[0]["uri"], uri,
            "the URI before the rewrite to /404.html"
        );
    }

    #[tokio::test]
    async fn request_logging_health_check_logs_at_debug() {
        let handler = handler("toml/handler.toml", |general| General {
            health: true,
            ..general
        });
        for path in ["/livez", "/readyz"] {
            let req = Request::get(format!("http://localhost{path}"))
                .body(())
                .unwrap();

            let (status, lines) = request_log_lines(&handler, req).await;

            assert_eq!(status, 200, "{path}");
            assert_eq!(lines.len(), 1, "{path}: {lines:?}");
            assert_eq!(lines[0]["level"], "DEBUG", "{path}");
            assert_eq!(lines[0]["status"], 200, "{path}");
        }
    }

    /// Sends a request with `X-Real-IP` and `X-Forwarded-For` from `REMOTE_ADDR`.
    async fn proxy_request_log_line(trusted_proxies: Vec<IpAddr>) -> serde_json::Value {
        let handler = handler("toml/handler.toml", |general| General {
            log_x_real_ip: true,
            log_forwarded_for: true,
            trusted_proxies,
            ..general
        });
        let req = Request::get("http://localhost/index.html")
            .header("X-Real-IP", "203.0.113.7")
            .header("X-Forwarded-For", "203.0.113.195, 198.51.100.1")
            .body(())
            .unwrap();

        let (_, mut lines) = request_log_lines(&handler, req).await;
        assert_eq!(lines.len(), 1, "{lines:?}");
        lines.remove(0)
    }

    #[tokio::test]
    async fn request_logging_trusted_proxy_logs_real_client_ips() {
        let proxy = REMOTE_ADDR.parse::<SocketAddr>().unwrap().ip();

        let line = proxy_request_log_line(vec![proxy]).await;

        assert_eq!(line["x_real_ip"], "203.0.113.7");
        assert_eq!(
            line["real_remote_ip"], "203.0.113.195",
            "first X-Forwarded-For entry"
        );
    }

    #[tokio::test]
    async fn request_logging_untrusted_proxy_omits_real_client_ips() {
        let line = proxy_request_log_line(vec!["192.0.2.1".parse().unwrap()]).await;

        assert!(line.get("x_real_ip").is_none(), "{line}");
        assert!(line.get("real_remote_ip").is_none(), "{line}");
    }
}
