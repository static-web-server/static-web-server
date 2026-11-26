#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

// Integration tests for the default per-request log line and the `error_page` line,
// whose fields follow the OpenTelemetry semantic conventions.
//
// Each test captures the JSON events of its own thread through a scoped
// `tracing` subscriber (`#[tokio::test]` runs on one thread), so the tests
// neither install nor depend on the global subscriber.

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use hyper::{Method, Request};
    use std::io::Write;
    use std::net::{IpAddr, SocketAddr};
    use std::sync::{Arc, Mutex};

    use static_web_server::handler::RequestHandler;
    use static_web_server::settings::cli::General;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    const RESPONSE_TARGET: &str = "static_web_server::log_addr";
    const ERROR_PAGE_TARGET: &str = "static_web_server::error_page";

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
        fn output(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    fn handler(fixture: &str, general: impl FnOnce(General) -> General) -> RequestHandler {
        let opts = fixture_settings(fixture);
        fixture_req_handler(fixture_req_handler_opts(
            general(opts.general),
            opts.advanced,
        ))
    }

    /// The response of a request, with all the JSON events logged for it.
    struct Logged {
        status: u16,
        body_len: usize,
        lines: Vec<serde_json::Value>,
    }

    impl Logged {
        /// The lines of `target`.
        fn lines_of(&self, target: &str) -> Vec<&serde_json::Value> {
            self.lines
                .iter()
                .filter(|l| l["target"] == target)
                .collect()
        }

        /// The one `response` line of the request.
        fn response_line(&self) -> &serde_json::Value {
            let lines = self.lines_of(RESPONSE_TARGET);
            assert_eq!(lines.len(), 1, "one log line per request: {:?}", self.lines);
            lines[0]
        }
    }

    /// Sends `req` from `remote_addr` through `handler`, reads the response body, and returns
    /// the response with the JSON events logged meanwhile.
    async fn request_from(
        handler: &RequestHandler,
        mut req: Request<()>,
        remote_addr: &str,
    ) -> Logged {
        let capture = Capture::default();
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(move || writer.clone())
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let remote_addr = Some(remote_addr.parse::<SocketAddr>().unwrap());
        let resp = handler
            .handle(&mut req, remote_addr)
            .await
            .expect("unexpected handler error");
        let status = resp.status().as_u16();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        drop(guard);

        let lines = capture
            .output()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON"))
            .collect();
        Logged {
            status,
            body_len: body.len(),
            lines,
        }
    }

    async fn request(handler: &RequestHandler, req: Request<()>) -> Logged {
        request_from(handler, req, REMOTE_ADDR).await
    }

    fn remote_ip() -> String {
        REMOTE_ADDR.parse::<SocketAddr>().unwrap().ip().to_string()
    }

    /// The field names of `line`, without the timestamp.
    fn field_names(line: &serde_json::Value) -> Vec<&str> {
        let mut names: Vec<&str> = line
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .filter(|k| *k != "timestamp")
            .collect();
        names.sort_unstable();
        names
    }

    #[tokio::test]
    async fn request_logging_response_line_uses_semantic_convention_names() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::get("http://localhost/missing.css")
            .header("user-agent", "curl/8.7.1")
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(logged.status, 404);
        let line = logged.response_line();
        assert_eq!(line["level"], "INFO");
        assert_eq!(line["message"], "response");
        assert_eq!(line["http.request.method"], "GET");
        assert_eq!(line["url.path"], "/missing.css");
        assert_eq!(line["http.response.status_code"], 404);
        assert_eq!(line["http.response.body.size"], logged.body_len as u64);
        assert!(
            line["duration"].as_f64().is_some_and(|d| d >= 0.0),
            "{line}"
        );
        assert_eq!(line["user_agent.original"], "curl/8.7.1");
        assert_eq!(
            field_names(line),
            [
                "duration",
                "http.request.method",
                "http.response.body.size",
                "http.response.status_code",
                "level",
                "message",
                "target",
                "url.path",
                "user_agent.original",
            ],
            "no address without its option, no former field names: {line}"
        );
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

            let logged = request(&handler, req).await;

            assert_eq!(logged.status, 200);
            let line = logged.response_line();
            assert_eq!(line["level"], "INFO");
            assert_eq!(line["http.request.method"], method.as_str());
            assert_eq!(
                line["url.path"], "/index.html",
                "the path, without the authority"
            );
            assert!(line.get("url.query").is_none(), "{line}");
            assert_eq!(line["http.response.status_code"], 200);
            assert_eq!(line["client.address"], remote_ip());
            assert_eq!(line["network.peer.address"], remote_ip());
        }
    }

    #[tokio::test]
    async fn request_logging_body_size_counts_the_body_sent() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::get("http://localhost/index.html")
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(logged.status, 200);
        let file_len = std::fs::metadata("docker/public/index.html").unwrap().len();
        assert_eq!(logged.body_len as u64, file_len);
        assert_eq!(logged.response_line()["http.response.body.size"], file_len);
    }

    #[cfg(feature = "compression-gzip")]
    #[tokio::test]
    async fn request_logging_body_size_counts_the_compressed_bytes() {
        let handler = handler("toml/handler.toml", |general| General {
            compression: true,
            ..general
        });
        let req = Request::get("http://localhost/index.html")
            .header("accept-encoding", "gzip")
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(logged.status, 200);
        let file_len = std::fs::metadata("docker/public/index.html").unwrap().len();
        assert!(
            (logged.body_len as u64) < file_len,
            "the body is compressed"
        );
        assert_eq!(
            logged.response_line()["http.response.body.size"],
            logged.body_len as u64
        );
    }

    #[tokio::test]
    async fn request_logging_body_size_of_a_range_is_the_part_sent() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::get("http://localhost/index.html")
            .header("range", "bytes=0-99")
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(logged.status, 206);
        assert_eq!(logged.body_len, 100);
        assert_eq!(logged.response_line()["http.response.body.size"], 100);
    }

    #[tokio::test]
    async fn request_logging_body_size_is_omitted_without_a_body() {
        let handler = handler("toml/handler.toml", |general| general);
        let head = Request::head("http://localhost/index.html")
            .body(())
            .unwrap();
        let options = Request::builder()
            .method(Method::OPTIONS)
            .uri("http://localhost/index.html")
            .body(())
            .unwrap();
        let not_modified = Request::get("http://localhost/index.html")
            .header("if-modified-since", "Fri, 01 Jan 2100 00:00:00 GMT")
            .body(())
            .unwrap();

        for (req, status) in [(head, 200), (options, 204), (not_modified, 304)] {
            let logged = request(&handler, req).await;

            assert_eq!(logged.status, status);
            let line = logged.response_line();
            assert_eq!(line["http.response.status_code"], status);
            assert!(line.get("http.response.body.size").is_none(), "{line}");
            assert!(line["duration"].is_f64(), "{line}");
        }
    }

    #[tokio::test]
    async fn request_logging_bytes_token_is_a_dash_without_a_body() {
        let handler = handler("toml/handler.toml", |general| General {
            access_log_format: "%{status} %{bytes}".to_owned(),
            ..general
        });
        let head = Request::head("http://localhost/index.html")
            .body(())
            .unwrap();
        let options = Request::builder()
            .method(Method::OPTIONS)
            .uri("http://localhost/index.html")
            .body(())
            .unwrap();
        let not_modified = Request::get("http://localhost/index.html")
            .header("if-modified-since", "Fri, 01 Jan 2100 00:00:00 GMT")
            .body(())
            .unwrap();

        for (req, line) in [(head, "200 -"), (options, "204 -"), (not_modified, "304 -")] {
            let logged = request(&handler, req).await;

            let access = logged.lines_of("static_web_server::access_logs");
            assert_eq!(access.len(), 1, "{:?}", logged.lines);
            assert_eq!(access[0]["message"], line);
        }
    }

    #[tokio::test]
    async fn request_logging_url_query_without_the_question_mark() {
        let handler = handler("toml/handler.toml", |general| general);
        let cases = [
            (
                "http://localhost/index.html?v=1&w=a%20b",
                Some("v=1&w=a%20b"),
            ),
            ("http://localhost/index.html?", None),
            ("http://localhost/index.html", None),
        ];
        for (uri, query) in cases {
            let req = Request::get(uri).body(()).unwrap();

            let logged = request(&handler, req).await;

            let line = logged.response_line();
            assert_eq!(line["url.path"], "/index.html", "{uri}");
            assert_eq!(
                line.get("url.query").and_then(|q| q.as_str()),
                query,
                "{uri}"
            );
        }
    }

    #[tokio::test]
    async fn request_logging_user_agent_is_capped_at_4_kib() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::get("http://localhost/index.html")
            .header("user-agent", "a".repeat(64 * 1024))
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(
            logged.response_line()["user_agent.original"],
            format!("{}...", "a".repeat(4096))
        );
    }

    #[tokio::test]
    async fn request_logging_duration_is_in_seconds_rounded_to_microseconds() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::get("http://localhost/index.html")
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;

        let duration = logged.response_line()["duration"].as_f64().unwrap();
        assert!((0.0..1.0).contains(&duration), "{duration}");
        assert_eq!(
            duration,
            format!("{duration:.6}").parse::<f64>().unwrap(),
            "the `%{{duration}}` value"
        );
    }

    #[tokio::test]
    async fn request_logging_error_page_line_uses_the_url_fields() {
        let handler = handler("toml/handler.toml", |general| general);
        let cases = [
            ("https://localhost:8443/missing.css?v=1", Some("v=1")),
            ("http://localhost/missing.css?", None),
        ];
        for (uri, query) in cases {
            let req = Request::get(uri).body(()).unwrap();

            let logged = request(&handler, req).await;

            let lines: Vec<_> = logged
                .lines_of(ERROR_PAGE_TARGET)
                .into_iter()
                .filter(|l| l["level"] == "WARN")
                .collect();
            assert_eq!(lines.len(), 1, "{:?}", logged.lines);
            let line = lines[0];
            assert_eq!(line["http.request.method"], "GET");
            assert_eq!(line["url.path"], "/missing.css", "{uri}");
            assert_eq!(
                line.get("url.query").and_then(|q| q.as_str()),
                query,
                "{uri}"
            );
            assert_eq!(line["http.response.status_code"], 404);
            assert_eq!(line["error"], "Not Found");
            for former in ["method", "uri", "status"] {
                assert!(line.get(former).is_none(), "{former}: {line}");
            }
        }
    }

    #[tokio::test]
    async fn request_logging_disallowed_method_logs_status_405() {
        let handler = handler("toml/handler.toml", |general| general);
        let req = Request::builder()
            .method(Method::PATCH)
            .uri("http://localhost/index.html")
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(logged.status, 405);
        let line = logged.response_line();
        assert_eq!(line["http.request.method"], "PATCH");
        assert_eq!(line["http.response.status_code"], 405);
    }

    #[tokio::test]
    async fn request_logging_redirect_logs_status_301() {
        let handler = handler("toml/redirects.toml", |general| General {
            root: "tests/fixtures/public".into(),
            ..general
        });
        let uri = "http://localhost/pages/about.html";
        let req = Request::get(uri).body(()).unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(logged.status, 301);
        let line = logged.response_line();
        assert_eq!(line["url.path"], "/pages/about.html");
        assert_eq!(line["http.response.status_code"], 301);
    }

    #[tokio::test]
    async fn request_logging_rewrite_logs_the_uri_sent_by_the_client() {
        let handler = handler("toml/rewrites.toml", |general| general);
        let uri = "http://localhost/some/error-page.html";
        let req = Request::get(uri).body(()).unwrap();

        let logged = request(&handler, req).await;

        assert_eq!(logged.status, 200);
        assert_eq!(
            logged.response_line()["url.path"],
            "/some/error-page.html",
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

            let logged = request(&handler, req).await;

            assert_eq!(logged.status, 200, "{path}");
            let line = logged.response_line();
            assert_eq!(line["level"], "DEBUG", "{path}");
            assert_eq!(line["http.response.status_code"], 200, "{path}");
        }
    }

    #[tokio::test]
    async fn request_logging_pretty_format_uses_the_same_names() {
        let handler = handler("toml/handler.toml", |general| general);
        let capture = Capture::default();
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let mut req = Request::get("http://localhost/index.html?v=1")
            .body(())
            .unwrap();
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());
        drop(handler.handle(&mut req, remote_addr).await.unwrap());
        drop(guard);

        let output = capture.output();
        let line = output
            .lines()
            .find(|l| l.contains(RESPONSE_TARGET))
            .unwrap_or_else(|| panic!("no response line:\n{output}"));
        for field in [
            "http.request.method=\"GET\"",
            "url.path=\"/index.html\"",
            "url.query=\"v=1\"",
            "http.response.status_code=200",
        ] {
            assert!(line.contains(field), "{field}: {line}");
        }
    }

    /// The proxy between the client and the one at `REMOTE_ADDR`.
    const INNER_PROXY: &str = "198.51.100.1";

    /// The flags that enable the addresses of the default line.
    #[derive(Clone, Copy)]
    struct AddressFlags {
        remote_address: bool,
        x_real_ip: bool,
        forwarded_for: bool,
    }

    const ALL_ADDRESSES: AddressFlags = AddressFlags {
        remote_address: true,
        x_real_ip: true,
        forwarded_for: true,
    };

    /// Sends a request with `X-Real-IP` and `X-Forwarded-For` from `REMOTE_ADDR`.
    ///
    /// The client `203.0.113.195` sends `X-Forwarded-For: 6.6.6.6`, `INNER_PROXY`
    /// appends the client to that line, and `REMOTE_ADDR` adds a line with `INNER_PROXY`.
    async fn proxy_request_log_line(
        trusted_proxies: Vec<IpAddr>,
        flags: AddressFlags,
    ) -> serde_json::Value {
        let handler = handler("toml/handler.toml", |general| General {
            log_remote_address: flags.remote_address,
            log_x_real_ip: flags.x_real_ip,
            log_forwarded_for: flags.forwarded_for,
            trusted_proxies,
            ..general
        });
        let req = Request::get("http://localhost/index.html")
            .header("X-Real-IP", "203.0.113.7")
            .header("X-Forwarded-For", "6.6.6.6, 203.0.113.195")
            .header("X-Forwarded-For", INNER_PROXY)
            .body(())
            .unwrap();

        let logged = request(&handler, req).await;
        let line = logged.response_line().clone();
        for former in ["remote_addr", "x_real_ip", "real_remote_ip"] {
            assert!(line.get(former).is_none(), "{former}: {line}");
        }
        line
    }

    fn trusted_chain() -> Vec<IpAddr> {
        let proxy = REMOTE_ADDR.parse::<SocketAddr>().unwrap().ip();
        vec![proxy, INNER_PROXY.parse().unwrap()]
    }

    #[tokio::test]
    async fn request_logging_trusted_proxy_forwarded_for_wins_over_x_real_ip() {
        let line = proxy_request_log_line(trusted_chain(), ALL_ADDRESSES).await;

        assert_eq!(
            line["client.address"], "203.0.113.195",
            "rightmost X-Forwarded-For entry that is not a trusted proxy"
        );
        assert_eq!(line["network.peer.address"], remote_ip());
    }

    #[tokio::test]
    async fn request_logging_peer_address_only_with_log_remote_address() {
        let flags = AddressFlags {
            remote_address: false,
            ..ALL_ADDRESSES
        };

        let line = proxy_request_log_line(trusted_chain(), flags).await;

        assert_eq!(line["client.address"], "203.0.113.195");
        assert!(line.get("network.peer.address").is_none(), "{line}");
    }

    #[tokio::test]
    async fn request_logging_proxy_headers_only_with_their_flags() {
        let flags = AddressFlags {
            remote_address: false,
            x_real_ip: false,
            forwarded_for: false,
        };

        let line = proxy_request_log_line(trusted_chain(), flags).await;

        assert!(line.get("client.address").is_none(), "{line}");
        assert!(line.get("network.peer.address").is_none(), "{line}");
    }

    #[tokio::test]
    async fn request_logging_unlisted_inner_proxy_is_logged_as_the_client() {
        let proxy = REMOTE_ADDR.parse::<SocketAddr>().unwrap().ip();

        let line = proxy_request_log_line(vec![proxy], ALL_ADDRESSES).await;

        assert_eq!(line["client.address"], INNER_PROXY);
    }

    #[tokio::test]
    async fn request_logging_untrusted_peer_is_the_client() {
        for trusted_proxies in [vec![], vec!["192.0.2.1".parse().unwrap()]] {
            let line = proxy_request_log_line(trusted_proxies.clone(), ALL_ADDRESSES).await;

            assert_eq!(line["client.address"], remote_ip(), "{trusted_proxies:?}");
            assert_eq!(line["network.peer.address"], remote_ip());

            let flags = AddressFlags {
                remote_address: false,
                ..ALL_ADDRESSES
            };
            let line = proxy_request_log_line(trusted_proxies.clone(), flags).await;
            assert!(line.get("client.address").is_none(), "{line}");
        }
    }

    #[tokio::test]
    async fn request_logging_ipv4_mapped_addresses_are_unmapped_in_both_lines() {
        let mapped_peer = "[::ffff:127.0.0.1]:1234";
        let req = || {
            Request::get("http://localhost/index.html")
                .header("X-Forwarded-For", "::ffff:203.0.113.195")
                .body(())
                .unwrap()
        };
        let general = |general: General| General {
            log_remote_address: true,
            trusted_proxies: vec!["127.0.0.1".parse().unwrap()],
            ..general
        };

        let default = handler("toml/handler.toml", general);
        let logged = request_from(&default, req(), mapped_peer).await;
        let line = logged.response_line();
        assert_eq!(line["client.address"], "127.0.0.1");
        assert_eq!(line["network.peer.address"], "127.0.0.1");

        let default = handler("toml/handler.toml", |g| General {
            log_forwarded_for: true,
            ..general(g)
        });
        let logged = request_from(&default, req(), mapped_peer).await;
        let line = logged.response_line();
        assert_eq!(line["client.address"], "203.0.113.195");
        assert_eq!(line["network.peer.address"], "127.0.0.1");

        let custom = handler("toml/handler.toml", |g| General {
            access_log_format: "%{remote_addr} %{forwarded_for}".to_owned(),
            ..general(g)
        });
        let logged = request_from(&custom, req(), mapped_peer).await;
        let access = logged.lines_of("static_web_server::access_logs");
        assert_eq!(access.len(), 1, "{:?}", logged.lines);
        assert_eq!(access[0]["message"], "127.0.0.1 203.0.113.195");
    }
}
