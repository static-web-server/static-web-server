---
name: testing
description: Write or review tests for the Static Web Server (SWS) project — unit tests, integration tests, test fixtures, and mocking strategies
---

# Testing Standards

Load this skill when writing, reviewing, or organizing tests — unit, integration, or fixture-based tests.

**When to load**: adding or editing any `#[test]` / `#[tokio::test]`, creating a new file under `tests/`, adding fixtures under `tests/fixtures/`, or reviewing a PR that changes test coverage.

## Testing Philosophy

- **Test behavior, not implementation**: Assert on HTTP status codes, response headers, and body content — not internal state. If refactoring without changing behavior breaks a test, the test is wrong
- **One scenario per test**: Each test verifies one request scenario. Multiple asserts are fine if they check the same logical outcome
- **Tests are documentation**: A test name describes the expected behavior. `compression_static_file_exists` is better than `test_compression_1`
- **Fast feedback**: Unit tests < 1ms. Integration tests < 100ms

## Rust Testing

### Unit Tests

- **Location**: `#[cfg(test)] mod tests { ... }` at the bottom of each source file. SWS follows this convention throughout `src/`
- **Naming**: `fn feature_scenario_description()`
- **Structure**: Arrange (setup response/fixture) → Act (call function) → Assert (check headers, status, body)
- **Cover edge cases**: Empty input, maximum input, invalid input, boundary values, unsupported methods
- **Use `assert_eq!` and `assert!`**: Prefer specific assertions over raw `assert!`

### Integration Tests

- **Location**: `tests/` directory at the crate root
- **Scope**: Each file tests one user-visible feature (e.g., `tests/compression.rs`, `tests/cors.rs`, `tests/dir_listing.rs`)
- **Test against real file fixtures**: Use `tests/fixtures/public/` for test files. Add new fixtures when testing new scenarios
- **Use the fixture infrastructure**: Import from `static_web_server::testing::fixtures`:
  - `fixture_settings("toml/handler_fixtures.toml")` — load TOML config
  - `fixture_req_handler_opts(general, advanced)` — build handler options
  - `fixture_req_handler(opts)` — create a request handler
- **Test with different HTTP methods**: Loop over GET, HEAD, OPTIONS and assert correct behavior per method

### Handler Tests

SWS's most common test pattern: create a handler, send a synthetic request, assert on the response:

```rust
use std::net::SocketAddr;
use hyper::{Method, Request, header::ACCEPT_ENCODING};
use static_web_server::testing::fixtures::*;
use static_web_server::settings::cli::General;

#[tokio::test]
async fn compression_static_file_exists() {
    let opts = fixture_settings("toml/handler_fixtures.toml");
    let general = General {
        compression_static: true,
        ..opts.general
    };
    let req_handler_opts = fixture_req_handler_opts(general, opts.advanced);
    let req_handler = fixture_req_handler(req_handler_opts);
    let remote_addr: Option<SocketAddr> = Some(REMOTE_ADDR.parse().unwrap());

    let mut req = Request::new(());
    *req.method_mut() = Method::GET;
    *req.uri_mut() = "http://localhost/index.htm".parse().unwrap();
    req.headers_mut().insert(ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());

    match req_handler.handle(&mut req, remote_addr).await {
        Ok(res) => {
            assert_eq!(res.status(), 200);
            assert_eq!(res.headers()["content-encoding"], "br");
            assert_eq!(res.headers()["vary"], "accept-encoding");
        }
        Err(err) => panic!("unexpected error: {err}"),
    }
}
```

`REMOTE_ADDR` (`"127.0.0.1:1234"`) is exported from `static_web_server::testing::fixtures`.

### Static File Tests

Tests in `tests/static_files.rs` call `static_files::handle()` directly with a `HandleOpts` struct. This tests the file-serving logic in isolation (without the full handler pipeline):

```rust
let result = static_files::handle(&HandleOpts {
    method: &Method::GET,
    headers: &HeaderMap::new(),
    base_path: &root_dir(),
    uri_path: "index.htm",
    index_files: &["index.htm"],
    // ... other opts
}).await;
```

### Cleaning Up

Integration tests using pre-existing fixtures under `tests/fixtures/` are read-only and need no cleanup. If a test creates temporary files (e.g., a temp upload directory), clean them up in a `Drop` handler or `#[tokio::test]` teardown step.

## Test Fixture Organization

```
tests/
  fixtures/
    public/           # Default test file tree
      index.html
      404.html
      assets/
        main.css
        main.css.zst  # Pre-compressed variant for static compression tests
    compression/       # Compression-specific test fixtures
    markdown/          # Markdown content-negotiation test fixtures
    toml/              # TOML config files for handler tests
    tls/               # TLS certificate/key test fixtures
```

## What to Test

- **Always test**: Public API surface, error cases, edge cases, HTTP status codes, response headers, supported/unsupported methods
- **Sometimes test**: Private functions with branching logic (3+ code paths) or performance-critical code (include benchmarks for the latter)
- **Don't test**: Trivial getters/setters, framework glue code, exact log message strings

## Run Commands

```bash
# Run all tests with all features
RUSTFLAGS="--cfg tokio_unstable" cargo test --tests --features="all"

# Run a specific test
cargo test --test compression -- compression_static_file_exists

# Run with trace logging visible
RUST_LOG=trace cargo test --test static_files -- --nocapture
```

## Checklist

- [ ] Do tests cover the happy path and at least one error path?
- [ ] Do integration tests clean up after themselves?
- [ ] Are test names descriptive?
- [ ] Are mocks used only for external dependencies (network), while filesystem access uses real test fixtures?
- [ ] Are test fixtures minimal (synthetic, small files)?
