#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

// Integration tests for file logging (`--log-file` / `SERVER_LOG_FILE` /
// `log-file`).
//
// These tests live in their own integration-test binary so the global
// `tracing` subscriber installed by `Settings::get_unparsed(true, ..)` does
// not interfere with other test files. Only ONE test in this file initializes
// the global subscriber (`writes_logs_to_file_through_settings_init`), since
// `tracing` allows the global subscriber to be set only once per process.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use static_web_server::Settings;

/// Poll `path` until it contains `needle` or `timeout` elapses. Returns the
/// final file content (which may not contain `needle` on timeout) so the
/// caller can use it in assertion failure messages.
///
/// File logging is asynchronous (the non-blocking writer drains on a
/// background thread), so callers must wait for the worker to flush rather
/// than reading immediately after emitting a log statement.
fn wait_for_log_content(path: &std::path::Path, needle: &str, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        let contents = std::fs::read_to_string(path).unwrap_or_default();
        if contents.contains(needle) {
            return contents;
        }
        if Instant::now() >= deadline {
            return contents;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// `--log-file` from CLI must be reflected in resolved settings.
#[test]
fn cli_log_file_is_reflected_in_settings() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("sws.log");

    // `log_init=false`: do NOT install the global subscriber from this test.
    let settings = Settings::get_unparsed(
        false,
        &[
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--port",
            "0",
            "--log-file",
            log_path.to_str().unwrap(),
        ],
    )
    .expect("settings must parse");

    assert_eq!(
        settings.general.log_file,
        Some(log_path),
        "--log-file should be reflected in resolved settings"
    );
}

/// When no `--log-file` is provided, the resolved setting is `None` so the
/// logger skips the file layer entirely.
#[test]
fn missing_log_file_resolves_to_none() {
    let settings = Settings::get_unparsed(
        false,
        &[
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--port",
            "0",
        ],
    )
    .expect("settings must parse");

    assert!(
        settings.general.log_file.is_none(),
        "log_file must default to None when --log-file is not set"
    );
}

/// TOML `log-file` under `[general]` is honored by settings parsing.
#[test]
fn toml_log_file_is_reflected_in_settings() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("from-toml.log");
    let config_path = tmp.path().join("sws.toml");

    // Use `Display::fmt` with proper TOML escaping by double-quoting the path
    // and replacing backslashes (Windows) / quotes — fine for tempdir paths.
    let toml = format!(
        "[general]\nroot = \"tests/fixtures/public\"\nlog-file = \"{}\"\n",
        log_path.display().to_string().replace('\\', "\\\\")
    );
    std::fs::write(&config_path, toml).expect("write toml");

    let settings = Settings::get_unparsed(
        false,
        &[
            "static-web-server",
            "--config-file",
            config_path.to_str().unwrap(),
        ],
    )
    .expect("settings must parse");

    assert_eq!(
        settings.general.log_file,
        Some(log_path),
        "log-file from TOML should populate settings.general.log_file"
    );
}

/// `Settings::get_unparsed(true, ..)` with `--log-file` installs the global
/// subscriber, opens the log file, and streams subsequent log events to it
/// through the non-blocking writer.
///
/// This is the ONLY test in this file that calls `log_init=true`, so it must
/// stay alone, `tracing`'s global subscriber can be installed only once.
#[test]
fn writes_logs_to_file_through_settings_init() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path: PathBuf = tmp.path().join("server.log");

    // Initialize the global subscriber with file output enabled. We pick a
    // generous level so the test event is captured regardless of the default.
    let _settings = Settings::get_unparsed(
        true,
        &[
            "static-web-server",
            "--root",
            "tests/fixtures/public",
            "--port",
            "0",
            "--log-level",
            "info",
            "--log-format",
            "json",
            "--log-file",
            log_path.to_str().unwrap(),
        ],
    )
    .expect("settings must parse and logger must initialize");

    // Emit a sentinel event that should reach the file via the non-blocking
    // worker thread.
    tracing::info!(test = "file-logging", "file-logging-integration-test");

    // Wait up to 3s for the background writer to flush the entry.
    let contents = wait_for_log_content(
        &log_path,
        "file-logging-integration-test",
        Duration::from_secs(3),
    );
    assert!(
        contents.contains("file-logging-integration-test"),
        "expected log message in file; got:\n{contents}"
    );

    // The chosen format is JSON, so every non-empty line must be valid JSON.
    for line in contents.lines().filter(|l| !l.is_empty()) {
        let parsed: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|err| panic!("log line is not valid JSON ({err}): {line}"));
        assert!(
            parsed.is_object(),
            "JSON log line must be an object: {line}"
        );
    }
}
