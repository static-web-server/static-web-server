#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

// Integration tests for the access log format (`--access-log-format` /
// `SERVER_ACCESS_LOG_FORMAT` / `access-log-format`).
//
// Each test captures the JSON events of its own thread through a scoped
// `tracing` subscriber (`#[tokio::test]` runs on one thread), so the tests
// neither install nor depend on the global subscriber.

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use hyper::{Method, Request, Version};
    use std::io::Write;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};

    use static_web_server::Settings;
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

    impl Capture {
        fn lines(&self) -> Vec<serde_json::Value> {
            let output = String::from_utf8(self.0.lock().unwrap().clone()).unwrap();
            output
                .lines()
                .map(|line| serde_json::from_str(line).expect("valid JSON"))
                .collect()
        }
    }

    /// Sends `req` through a handler built from `fixture` and `general`, reads the response
    /// body, and returns the response status, the body length and all JSON events emitted
    /// meanwhile. With `check`, also calls it with the events emitted before reading the body.
    async fn handle(
        fixture: &str,
        general: impl FnOnce(General) -> General,
        mut req: Request<()>,
        check: Option<fn(Vec<serde_json::Value>)>,
    ) -> (u16, usize, Vec<serde_json::Value>) {
        let opts = fixture_settings(fixture);
        let handler = fixture_req_handler(fixture_req_handler_opts(
            general(opts.general),
            opts.advanced,
        ));

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
        let resp = handler
            .handle(&mut req, remote_addr)
            .await
            .expect("unexpected handler error");
        if let Some(check) = check {
            check(capture.lines());
        }
        let status = resp.status().as_u16();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        drop(guard);

        (status, body.len(), capture.lines())
    }

    async fn log_lines(
        fixture: &str,
        general: impl FnOnce(General) -> General,
        req: Request<()>,
    ) -> (u16, Vec<serde_json::Value>) {
        let (status, _, lines) = handle(fixture, general, req, None).await;
        (status, lines)
    }

    fn with_format(format: &str) -> impl FnOnce(General) -> General {
        move |general| General {
            access_log_format: format.to_owned(),
            ..general
        }
    }

    fn access_log_lines(lines: &[serde_json::Value]) -> Vec<&serde_json::Value> {
        lines
            .iter()
            .filter(|l| l["target"] == "static_web_server::access_logs")
            .collect()
    }

    fn settings(args: &[&str]) -> Settings {
        let mut argv = vec!["static-web-server"];
        argv.extend_from_slice(args);
        Settings::get_unparsed(false, &argv).expect("settings must parse")
    }

    #[tokio::test]
    async fn access_log_format_set_replaces_the_default_request_line() {
        let format = r#"%{remote_addr} "%{method} %{uri} %{version}" %{status} "%{user_agent}""#;
        let ip = REMOTE_ADDR.parse::<SocketAddr>().unwrap().ip();

        for method in [Method::GET, Method::HEAD] {
            let req = Request::builder()
                .method(method.clone())
                .uri("http://localhost/index.html")
                .header("user-agent", "curl/8.0")
                .body(())
                .unwrap();

            let (status, lines) = log_lines("toml/handler.toml", with_format(format), req).await;

            assert_eq!(status, 200);
            let access = access_log_lines(&lines);
            assert_eq!(access.len(), 1, "{lines:?}");
            assert_eq!(access[0]["level"], "INFO");
            assert_eq!(
                access[0]["message"],
                format!(r#"{ip} "{method} /index.html HTTP/1.1" 200 "curl/8.0""#)
            );
            assert!(
                lines
                    .iter()
                    .all(|l| l["target"] != "static_web_server::log_addr"),
                "the default request line must not be logged: {lines:?}"
            );
        }
    }

    /// Requests `/index.html` with `accept-encoding`, and returns the logged `%{bytes}` and the
    /// length of the body sent.
    async fn bytes_sent(encoding: &str) -> (serde_json::Value, usize) {
        let req = Request::get("http://localhost/index.html")
            .header("accept-encoding", encoding)
            .body(())
            .unwrap();

        let (status, body_len, lines) =
            handle("toml/handler.toml", with_format("%{bytes}"), req, None).await;

        assert_eq!(status, 200);
        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        (access[0]["message"].clone(), body_len)
    }

    #[tokio::test]
    async fn access_log_format_bytes_logs_the_body_sent() {
        let file_len = std::fs::metadata("docker/public/index.html").unwrap().len();

        let (logged, body_len) = bytes_sent("identity").await;

        assert_eq!(body_len as u64, file_len);
        assert_eq!(logged, body_len.to_string());
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn access_log_format_bytes_logs_the_compressed_body_sent() {
        let file_len = std::fs::metadata("docker/public/index.html").unwrap().len();

        let (logged, body_len) = bytes_sent("gzip").await;

        assert!((body_len as u64) < file_len, "compressed on the fly");
        assert_eq!(logged, body_len.to_string());
    }

    #[tokio::test]
    async fn access_log_format_bytes_is_a_dash_without_a_body() {
        let head = Request::head("http://localhost/index.html")
            .body(())
            .unwrap();
        let not_modified = Request::get("http://localhost/index.html")
            .header("if-modified-since", "Fri, 01 Jan 2100 00:00:00 GMT")
            .body(())
            .unwrap();

        for (req, expected) in [(head, "200 -"), (not_modified, "304 -")] {
            let (_, lines) =
                log_lines("toml/handler.toml", with_format("%{status} %{bytes}"), req).await;

            let access = access_log_lines(&lines);
            assert_eq!(access.len(), 1, "{lines:?}");
            assert_eq!(access[0]["message"], expected);
        }
    }

    #[tokio::test]
    async fn access_log_format_logs_once_the_body_is_dropped() {
        let req = Request::get("http://localhost/index.html")
            .body(())
            .unwrap();
        let before_body = |lines: Vec<serde_json::Value>| {
            assert!(access_log_lines(&lines).is_empty(), "{lines:?}");
        };

        let (_, _, lines) = handle(
            "toml/handler.toml",
            with_format("%{status}"),
            req,
            Some(before_body),
        )
        .await;

        assert_eq!(access_log_lines(&lines).len(), 1, "{lines:?}");
    }

    #[tokio::test]
    async fn access_log_format_unset_keeps_the_default_request_line() {
        let req = Request::get("http://localhost/index.html")
            .body(())
            .unwrap();

        let (status, lines) = log_lines("toml/handler.toml", |general| general, req).await;

        assert_eq!(status, 200);
        assert!(access_log_lines(&lines).is_empty(), "{lines:?}");
        assert!(
            lines
                .iter()
                .any(|l| l["target"] == "static_web_server::log_addr"),
            "{lines:?}"
        );
    }

    #[tokio::test]
    async fn access_log_format_rewrite_logs_the_uri_sent_by_the_client() {
        let req = Request::get("http://localhost/some/error-page.html")
            .body(())
            .unwrap();

        let (status, lines) =
            log_lines("toml/rewrites.toml", with_format("%{uri} %{status}"), req).await;

        assert_eq!(status, 200);
        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(
            access[0]["message"], "/some/error-page.html 200",
            "the URI before the rewrite to /404.html"
        );
    }

    #[tokio::test]
    async fn access_log_format_uri_logs_the_path_and_query_only() {
        // Absolute-form, as `Uri` carries it over HTTP/2
        let req = Request::get("https://127.0.0.1:8443/index.html?v=1")
            .body(())
            .unwrap();

        let (status, lines) = log_lines("toml/handler.toml", with_format("%{uri}"), req).await;

        assert_eq!(status, 200);
        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(access[0]["message"], "/index.html?v=1");
    }

    #[tokio::test]
    async fn access_log_format_host_logs_the_http2_authority() {
        // HTTP/2 sends the host as `:authority`, which hyper keeps in the URI, not in `Host`
        let req = Request::get("https://example.com:8443/index.html")
            .version(Version::HTTP_2)
            .body(())
            .unwrap();

        let (_, lines) = log_lines("toml/handler.toml", with_format("%{host}"), req).await;

        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(access[0]["message"], "example.com:8443");
    }

    #[tokio::test]
    async fn access_log_format_host_prefers_the_uri_authority() {
        // An absolute-form target overrides `Host` (RFC 9112, 3.2.2), as in `virtual_hosts`
        let req = Request::get("http://vhost.example/index.html")
            .header("host", "other.example")
            .body(())
            .unwrap();

        let (_, lines) = log_lines("toml/handler.toml", with_format("%{host}"), req).await;

        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(access[0]["message"], "vhost.example");
    }

    #[tokio::test]
    async fn access_log_format_host_logs_the_host_sent_by_the_client() {
        let req = Request::get("/vhost/index.html")
            .header("host", "localhost:8787")
            .body(())
            .unwrap();

        let (_, lines) = log_lines(
            "toml/access_log_format_rewrites.toml",
            with_format("%{host} %{uri}"),
            req,
        )
        .await;

        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(
            access[0]["message"], "localhost:8787 /vhost/index.html",
            "the Host before the rewrite to other.example"
        );
    }

    fn proxied_request() -> Request<()> {
        Request::get("http://localhost/index.html")
            .header("x-real-ip", "203.0.113.7")
            .header("x-forwarded-for", "203.0.113.195, 10.0.0.1")
            .body(())
            .unwrap()
    }

    #[tokio::test]
    async fn access_log_format_proxy_tokens_need_no_log_flags() {
        let format = "%{x_real_ip} %{forwarded_for}";

        let (_, lines) =
            log_lines("toml/handler.toml", with_format(format), proxied_request()).await;

        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(access[0]["message"], "203.0.113.7 203.0.113.195");
    }

    #[tokio::test]
    async fn access_log_format_proxy_tokens_follow_trusted_proxies() {
        let format = "%{x_real_ip} %{forwarded_for}";
        let general = |general| General {
            trusted_proxies: vec!["10.0.0.1".parse().unwrap()],
            ..with_format(format)(general)
        };

        let (_, lines) = log_lines("toml/handler.toml", general, proxied_request()).await;

        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(
            access[0]["message"], "- -",
            "the remote address is not a trusted proxy"
        );
    }

    #[tokio::test]
    async fn access_log_format_client_values_are_escaped() {
        let req = Request::get("http://localhost/missing.css")
            .header("user-agent", r#"x" 200 "forged"#)
            .header("referer", "a\\b")
            .body(())
            .unwrap();
        let format = r#"%{status} "%{user_agent}" "%{referer}""#;

        let (status, lines) = log_lines("toml/handler.toml", with_format(format), req).await;

        assert_eq!(status, 404);
        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(
            access[0]["message"],
            r#"404 "x\x22 200 \x22forged" "a\x5Cb""#
        );
    }

    #[tokio::test]
    async fn access_log_format_long_client_values_are_truncated() {
        let req = Request::get("http://localhost/index.html")
            .header("user-agent", "a".repeat(64 * 1024))
            .body(())
            .unwrap();

        let (_, lines) = log_lines("toml/handler.toml", with_format("%{user_agent}"), req).await;

        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(
            access[0]["message"],
            format!("{}...", "a".repeat(4096)),
            "cut at 4 KiB"
        );
    }

    #[tokio::test]
    async fn access_log_format_health_check_logs_at_debug() {
        let req = Request::get("http://localhost/livez").body(()).unwrap();
        let general = |general| General {
            health: true,
            ..with_format("%{uri} %{status}")(general)
        };

        let (status, lines) = log_lines("toml/handler.toml", general, req).await;

        assert_eq!(status, 200);
        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(access[0]["level"], "DEBUG");
    }

    #[test]
    fn access_log_format_default_is_empty() {
        let settings = settings(&["--root", "tests/fixtures/public"]);
        assert_eq!(settings.general.access_log_format, "");
    }

    #[test]
    fn access_log_format_cli_flag_sets_the_format() {
        let settings = settings(&[
            "--root",
            "tests/fixtures/public",
            "--access-log-format",
            "%{method} %{status}",
        ]);
        assert_eq!(settings.general.access_log_format, "%{method} %{status}");
    }

    #[test]
    fn access_log_format_toml_key_overrides_the_cli_flag() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config_path = tmp.path().join("sws.toml");
        std::fs::write(
            &config_path,
            "[general]\nroot = \"tests/fixtures/public\"\naccess-log-format = \"%{uri} %{status}\"\n",
        )
        .expect("write toml");

        let settings = settings(&[
            "--config-file",
            config_path.to_str().unwrap(),
            "--access-log-format",
            "%{method}",
        ]);
        assert_eq!(settings.general.access_log_format, "%{uri} %{status}");
    }
}
