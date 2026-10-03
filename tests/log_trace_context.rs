#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

// Integration tests for the trace context in JSON log lines
// (`--log-trace-context` / `SERVER_LOG_TRACE_CONTEXT` / `log-trace-context`).
//
// These tests live in their own integration-test binary because one of them
// installs the global `tracing` subscriber, which can be done only once per
// process.

use hyper::Request;
use std::net::SocketAddr;
use std::path::Path;
use std::time::{Duration, Instant};

use static_web_server::Settings;
use static_web_server::testing::fixtures::{
    REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts,
};

const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

/// Poll `path` until every uri in `uris` appears at least `count` times and
/// return the file content, or panic when `timeout` elapses. File logging is
/// asynchronous (non-blocking writer).
fn wait_for_log_lines(path: &Path, uris: &[&str], count: usize, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        let contents = std::fs::read_to_string(path).unwrap_or_default();
        let complete = uris
            .iter()
            .all(|uri| contents.matches(&format!(r#""uri":"{uri}""#)).count() >= count);
        if complete {
            return contents;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {count} log lines per uri {uris:?}:\n{contents}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn log_trace_context_is_disabled_by_default() {
    let settings = Settings::get_unparsed(
        false,
        &["static-web-server", "--root", "tests/fixtures/public"],
    )
    .expect("settings must parse");
    assert!(!settings.general.log_trace_context);
}

#[test]
fn log_trace_context_cli_flag() {
    let settings = Settings::get_unparsed(
        false,
        &[
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--log-trace-context",
        ],
    )
    .expect("settings must parse");
    assert!(settings.general.log_trace_context);
}

#[test]
fn log_trace_context_toml_option() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config_path = tmp.path().join("sws.toml");
    std::fs::write(
        &config_path,
        "[general]\nroot = \"tests/fixtures/public\"\nlog-trace-context = true\n",
    )
    .expect("write toml");

    let settings = Settings::get_unparsed(
        false,
        &[
            "static-web-server",
            "--config-file",
            config_path.to_str().unwrap(),
        ],
    )
    .expect("settings must parse");
    assert!(settings.general.log_trace_context);
}

#[test]
fn log_trace_context_requires_json_format() {
    let result = Settings::get_unparsed(
        false,
        &[
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--log-format",
            "pretty",
            "--log-trace-context",
        ],
    );
    let err = result.err().expect("pretty format must be rejected");
    assert!(
        err.to_string()
            .contains("--log-trace-context requires --log-format=json"),
        "unexpected error: {err}"
    );
}

/// The JSON log lines emitted by the request handler carry the trace context fields only
/// when the option is enabled and the request has a valid `traceparent`.
///
/// This is the ONLY test in this file that installs the global subscriber.
#[tokio::test]
async fn log_trace_context_in_request_log_lines() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("server.log");

    let settings = Settings::get_unparsed(
        true,
        &[
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--log-level",
            "info",
            "--log-format",
            "json",
            "--log-file",
            log_path.to_str().unwrap(),
            "--log-trace-context",
        ],
    )
    .expect("settings must parse and logger must initialize");

    let enabled = fixture_req_handler(fixture_req_handler_opts(
        settings.general,
        settings.advanced,
    ));
    // Default settings, without `--log-trace-context`
    let defaults = Settings::get_unparsed(
        false,
        &["static-web-server", "--root", "tests/fixtures/public"],
    )
    .expect("settings must parse");
    let disabled = fixture_req_handler(fixture_req_handler_opts(
        defaults.general,
        defaults.advanced,
    ));
    let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

    let uppercase = TRACEPARENT.to_uppercase();
    let zero_trace_id = "00-00000000000000000000000000000000-00f067aa0ba902b7-01";
    let requests = [
        (&enabled, "/traced.css", Some(TRACEPARENT)),
        (&enabled, "/no-header.css", None),
        (&enabled, "/uppercase.css", Some(uppercase.as_str())),
        (&enabled, "/zero-trace-id.css", Some(zero_trace_id)),
        (&disabled, "/disabled.css", Some(TRACEPARENT)),
    ];
    for (handler, path, traceparent) in requests {
        let mut builder = Request::get(path);
        if let Some(value) = traceparent {
            builder = builder.header("traceparent", value);
        }
        let mut req = builder.body(()).unwrap();
        let res = handler.handle(&mut req, remote_addr).await.unwrap();
        assert_eq!(res.status(), 404, "{path}");
    }

    let uris: Vec<&str> = requests.iter().map(|(_, path, _)| *path).collect();
    // The `incoming request` line and the `404` warning of each request
    let contents = wait_for_log_lines(&log_path, &uris, 2, Duration::from_secs(3));
    let lines: Vec<serde_json::Value> = contents
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid JSON log line"))
        .collect();

    for (_, path, _) in requests {
        let request_lines: Vec<_> = lines.iter().filter(|l| l["uri"] == path).collect();
        assert!(
            request_lines
                .iter()
                .any(|l| l["message"] == "incoming request"),
            "{path}: no `incoming request` line:\n{contents}"
        );
        assert!(
            request_lines
                .iter()
                .any(|l| l["target"] == "static_web_server::error_page"),
            "{path}: no `error_page` line:\n{contents}"
        );

        for line in request_lines {
            if path == "/traced.css" {
                assert_eq!(line["trace_id"], "4bf92f3577b34da6a3ce929d0e0e4736");
                assert_eq!(line["span_id"], "00f067aa0ba902b7");
                assert_eq!(line["trace_flags"], "01");
            } else {
                assert!(line.get("trace_id").is_none(), "{line}");
                assert!(line.get("span_id").is_none(), "{line}");
                assert!(line.get("trace_flags").is_none(), "{line}");
            }
        }
    }
}
