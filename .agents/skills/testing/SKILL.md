---
name: testing
description: Write or review tests for the Static Web Server (SWS) project — unit tests, handler and static-file integration tests, TOML and file fixtures, property tests, fuzz targets, and benchmarks. Use when adding or editing any test, fixture, proptest, fuzz target, or bench, or when a test fails.
---

# Testing Standards

Every behavior change ships with a test that fails before the change and passes after it. Prove it: run the test on the base before the fix.

**When to load**: adding or editing `#[test]` / `#[tokio::test]`, creating a file under `tests/`, adding fixtures, writing a proptest, fuzz target, or bench, or diagnosing a failing test.

## Where Tests Go

| Kind | Location | Use for |
|------|----------|---------|
| Unit | `#[cfg(test)] mod tests` at the bottom of the source file | Pure functions and private logic (`control_headers::append_headers`, `compression::auto`, q-values) |
| Handler integration | `tests/<feature>.rs` via `RequestHandler::handle()` | Behavior that depends on pipeline order or several steps. Security fixes always get one |
| Static-file integration | `tests/static_files.rs` via `static_files::handle()` | Resolution, index files, ranges, pre-compressed variants, without the pipeline |
| Settings | `tests/settings.rs` | CLI/env/TOML parsing and merging |
| TLS / server | `tests/tls.rs`, `tests/idle_timeout.rs`, `tests/max_header_size.rs`, `tests/max_connections.rs` | Certificates in `tests/tls/`; connection limits |
| Property | `proptest!` blocks in source files | Parsers and path handling over arbitrary input |
| Fuzz | `fuzz/src/*.rs` (cargo-fuzz) | Untrusted-input parsers |
| Bench | `benches/*.rs` (Criterion/CodSpeed) | Hot-path functions |

Integration files start with the crate lints and wrap tests in a module:

```rust
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests { /* ... */ }
```

Gate tests on the features they need (`#[cfg(feature = "directory-listing")]`) and gate their imports too, or `cargo test --no-default-features` fails on unused imports.

## Handler Test Pattern

```rust
use hyper::{Request, header::ACCEPT_ENCODING};
use std::net::SocketAddr;
use static_web_server::settings::cli::General;
use static_web_server::testing::fixtures::{
    REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
};

#[tokio::test]
async fn compression_static_serves_brotli_variant() {
    let opts = fixture_settings("toml/handler_fixtures.toml");
    let general = General { compression_static: true, ..opts.general };
    let req_handler = fixture_req_handler(fixture_req_handler_opts(general, opts.advanced));
    let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

    let mut req = Request::new(());
    *req.uri_mut() = "http://localhost/index.htm".parse().unwrap();
    req.headers_mut().insert(ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());

    let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["content-encoding"], "br");
}
```

### Fixture Helpers (`src/testing.rs`)

- `fixture_settings(path)` loads `tests/fixtures/<path>` through the real parser (`Settings::get_unparsed`), so TOML `[general]` overrides CLI defaults as in production
- `fixture_req_handler_opts(general, advanced)` copies most `General` fields but **hard-codes** `cors: None`, `page_fallback: vec![]`, `memory_cache: None`. To test those, set the fields on the returned `RequestHandlerOpts` or call the module's `init()` first
- It does not run `server/opts.rs`, so `init()` side effects (compiled CORS, cache store, loaded error pages) are absent unless the test performs them
- A new `RequestHandlerOpts` field must be mapped here or the crate fails to compile

## Static File Test Pattern

```rust
let result = static_files::handle(&HandleOpts {
    method: &Method::GET,
    headers: &HeaderMap::new(),
    base_path: &PathBuf::from("tests/fixtures/public/"),
    uri_path: "index.htm",
    uri_query: None,
    index_files: &["index.htm"],
    #[cfg(feature = "mem-cache")]
    memory_cache: None,
    #[cfg(feature = "directory-listing")]
    dir_listing: false,
    #[cfg(feature = "directory-listing")]
    dir_listing_order: 6,
    #[cfg(feature = "directory-listing")]
    dir_listing_format: &DirListFmt::Html,
    #[cfg(feature = "directory-listing-download")]
    dir_listing_download: &[],
    redirect_trailing_slash: true,
    compression_static: false,
    etag: false,
    include_hidden: false,
    follow_symlinks: false,
})
.await;
// Ok(StaticFileResponse { resp, file_path, is_dir_listing }) or Err(StatusCode)
```

## Fixtures

```
tests/fixtures/
  public/        default root: index.htm (+ .br), index.html.gz, 404.html (+ .br), 50x.html, readme.md,
                 assets/ (index.html + .gz, main.css + .zst, main.js, tiny.js + .br),
                 symlink, "spécial-directöry.net/" (non-ASCII, spaces)
  compression/   large-test.html, above the 200-byte dynamic compression threshold
  markdown/      .md variants for --accept-markdown
  symlink/       targets for symlink policy tests
  empty/         empty root
  toml/          configs for fixture_settings("toml/<name>.toml")
tests/tls/       PEM certificates and keys (PKCS#1, PKCS#8, SEC1)
tests/toml/      sws.toml, memory_cache.toml
```

- Reuse `public/`; add a new file instead of changing an existing one, which can break unrelated tests
- Files under 200 bytes are never compressed dynamically; use `compression/` for that
- Tests that write files use `tempfile`. Paths are relative to the crate root

## Property Tests and Fuzzing

- `proptest` is used in `access_logs.rs`, `cors.rs`, `fs/path.rs`, `redirects.rs`, `markdown.rs`, `response/range.rs`, `exts/headers/quality_value.rs`, `directory_listing/download.rs`. Add one for any new parser of untrusted input
- Shrunk failures persist under `proptest-regressions/`; commit them with the fix
- Fuzz targets: `fuzz/src/{static_files,content_disposition,cors_origin,redirect_placeholders}.rs`; `cargo +nightly fuzz run <target>` from the repo root

## Benchmarks

`benches/` is a separate crate (`codspeed-criterion-compat`) run by `.github/workflows/codspeed.yml`. Add a bench when a change touches a hot-path function (`performance/SKILL.md`).

## Running Tests

```bash
cargo test --features all
cargo test --no-default-features
cargo test --features all --test compression                   # one integration file
cargo test --features all --test compression -- <test_name>
cargo test --features all --lib control_headers                # unit tests of one module
RUST_LOG=trace cargo test --features all --test static_files -- --nocapture
```

Tooling pitfalls (`RUSTFLAGS`, `--all-features`): `rust-conventions/SKILL.md`.

## What to Test

- **Always**: status, headers (presence and absence), body for small responses, `GET` vs `HEAD` vs `OPTIONS`, 405 for other methods, the feature-off path
- **Security**: traversal (`../`, `%2e%2e`, absolute paths), hidden files, symlinks with `follow_symlinks` on and off, non-ASCII paths, auth on every method
- **Config options**: default, CLI flag, env var, TOML key, and TOML overriding CLI
- **Skip**: trivial getters, exact log strings, hyper internals

## Naming and Style

- `fn <feature>_<scenario>_<expectation>()`, e.g. `cache_control_404_uses_no_cache`
- Arrange → act → assert; one scenario per test
- Message on non-obvious asserts: `assert_eq!(v, "br", "pre-compressed variant must win")`
- Loop over methods when behavior must hold for each

## Checklist

- [ ] The test fails without the change and passes with it
- [ ] Happy path and at least one error path
- [ ] Passes under `--features all` and `--no-default-features`
- [ ] New `RequestHandlerOpts` fields mapped in `testing.rs`
- [ ] New untrusted-input parsers have a proptest
- [ ] Fixtures added, not modified
