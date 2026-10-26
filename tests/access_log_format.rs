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
    use hyper::{Method, Request};
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

    /// Sends `req` through a handler built from `fixture` and `general`, and returns the
    /// response status and all JSON events emitted while handling it.
    async fn log_lines(
        fixture: &str,
        general: impl FnOnce(General) -> General,
        mut req: Request<()>,
    ) -> (u16, Vec<serde_json::Value>) {
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
            .map(|line| serde_json::from_str(line).expect("valid JSON"))
            .collect();
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
                format!(r#"{ip} "{method} http://localhost/index.html HTTP/1.1" 200 "curl/8.0""#)
            );
            assert!(
                lines
                    .iter()
                    .all(|l| l["target"] != "static_web_server::log_addr"),
                "the default request line must not be logged: {lines:?}"
            );
        }
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
        let uri = "http://localhost/some/error-page.html";
        let req = Request::get(uri).body(()).unwrap();

        let (status, lines) =
            log_lines("toml/rewrites.toml", with_format("%{uri} %{status}"), req).await;

        assert_eq!(status, 200);
        let access = access_log_lines(&lines);
        assert_eq!(access.len(), 1, "{lines:?}");
        assert_eq!(
            access[0]["message"],
            format!("{uri} 200"),
            "the URI before the rewrite to /404.html"
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
    async fn access_log_format_health_check_logs_at_debug() {
        let req = Request::get("http://localhost/health").body(()).unwrap();
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
