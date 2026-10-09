---
name: testing
description: Write or review tests for the Static Web Server (SWS) project — unit tests, handler and static-file integration tests, TOML and file fixtures, property tests, fuzz targets, and benchmarks. Use when adding or editing any test, fixture, proptest, fuzz target, or bench, or when a test fails.
---

# Testing Standards

Every behavior change ships with a test that fails before the change and passes after it.

**When to load**: adding or editing `#[test]` / `#[tokio::test]`, creating a file under `tests/`, adding fixtures under `tests/fixtures/`, writing a property test, fuzz target, or bench, or diagnosing a failing test.

## Where Tests Go

| Kind | Location | Use for |
|------|----------|---------|
| Unit | `#[cfg(test)] mod tests` at the bottom of the source file | Pure functions and private logic (`control_headers::append_headers`, `compression::auto`, q-value parsing) |
| Handler integration | `tests/<feature>.rs` via `RequestHandler::handle()` | Behavior that depends on pipeline order or multiple steps |
| Static-file integration | `tests/static_files.rs` via `static_files::handle()` | Path resolution, index files, ranges, pre-compressed variants, without the pipeline |
| Settings | `tests/settings.rs` | CLI/env/TOML parsing and merging |
| TLS / server | `tests/tls.rs` | Certificate formats in `tests/tls/` |
| Property | `proptest!` blocks in source files | Parsers and path handling over arbitrary input |
| Fuzz | `fuzz/src/*.rs` (cargo-fuzz) | Untrusted-input parsers |
| Bench | `benches/*.rs` (Criterion/CodSpeed) | Hot-path functions |

Integration test files start with the crate lint attributes and wrap tests in a module:

```rust
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    // ...
}
```

Gate tests on the features they need (`#[cfg(feature = "directory-listing")]`), and gate their imports too, or `cargo test --no-default-features` fails on unused imports.

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
    let general = General {
        compression_static: true,
        ..opts.general
    };
    let req_handler = fixture_req_handler(fixture_req_handler_opts(general, opts.advanced));
    let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

    let mut req = Request::new(());
    *req.uri_mut() = "http://localhost/index.htm".parse().unwrap();
    req.headers_mut()
        .insert(ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());

    let res = req_handler.handle(&mut req, remote_addr).await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["content-encoding"], "br");
}
```

### Fixture Helper Behavior (`src/testing.rs`)

- `fixture_settings(path)` loads `tests/fixtures/<path>` through the real settings parser (`Settings::get_unparsed`), so TOML `[general]` values override CLI defaults exactly as in production
- `fixture_req_handler_opts(general, advanced)` copies most `General` fields but **hard-codes** `cors: None`, `page_fallback: vec![]`, and `memory_cache: None`. To test CORS, fallback pages, or the memory cache, set those fields on the returned `RequestHandlerOpts` (or call the module's `init()`) before building the handler
- It does not run `server/opts.rs`, so `init()` side effects (compiled CORS config, cache store, loaded 404/50x pages) are absent unless the test performs them
- When you add a field to `RequestHandlerOpts`, add it to `fixture_req_handler_opts()` too

## Static File Test Pattern

```rust
let result = static_files::handle(&HandleOpts {
    method: &Method::GET,
    headers: &HeaderMap::new(),
    base_path: &PathBuf::from("tests/fixtures/public/"),
    uri_path: "index.htm",
    uri_query: None,
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
    index_files: &["index.htm"],
    follow_symlinks: false,
})
.await;
// Ok(StaticFileResponse { resp, file_path }) or Err(StatusCode)
```

## Fixtures

```
tests/
  fixtures/
    public/        default web root: index.htm (+ .br), index.html.gz, 404.html (+ .br), 50x.html,
                   readme.md, assets/ (main.css + .zst, tiny.js + .br, main.js),
                   symlink, "spécial-directöry.net/" (non-ASCII and space handling)
    compression/   large-test.html — above the 200-byte dynamic compression threshold
    markdown/      .md variants for --accept-markdown
    symlink/       targets for symlink policy tests
    empty/         empty root
    toml/          TOML configs loaded by fixture_settings("toml/<name>.toml")
  tls/             PEM certificates and keys (PKCS#1, PKCS#8, SEC1)
  toml/            sws.toml, memory_cache.toml
```

- Reuse `tests/fixtures/public/` when possible. Changing an existing fixture file can break unrelated tests; add a new file instead
- Files in `public/` under 200 bytes are never compressed dynamically. Use `compression/` for dynamic compression tests
- Tests that write files use `tempfile` (dev-dependency) so cleanup is automatic
- Paths are relative to the crate root; `cargo test` runs from there

## Property Tests and Fuzzing

- `proptest` is a dev-dependency used in `cors.rs`, `fs/path.rs`, `redirects.rs`, `markdown.rs`, `response/range.rs`, `exts/headers/quality_value.rs`, and `directory_listing/download.rs`. Add one for any new parser of untrusted input
- Shrunk failures persist under `proptest-regressions/`. Commit new regression files with the fix
- Fuzz targets live in `fuzz/` (`static_files`, `content_disposition`, `cors_origin`, `redirect_placeholders`): `cargo +nightly fuzz run <target>` from the repo root

## Benchmarks

`benches/` is a separate crate (Criterion via `codspeed-criterion-compat`) run on every PR by `.github/workflows/codspeed.yml`. Add a bench when a change touches a hot-path function. See `performance/SKILL.md`.

## Running Tests

```bash
cargo test --features all                              # everything
cargo test --no-default-features                       # strict feature build
cargo test --features all --test compression           # one integration file
cargo test --features all --test compression -- compression_static_file_exists
cargo test --features all --lib control_headers        # unit tests in one module
RUST_LOG=trace cargo test --features all --test static_files -- --nocapture
```

`.cargo/config.toml` sets `--cfg tokio_unstable`. An exported `RUSTFLAGS` overrides it and breaks `--features all`; append the flag instead: `RUSTFLAGS="$RUSTFLAGS --cfg tokio_unstable"`.

## What to Test

- **Always**: status codes, response headers (presence and absence), body for small responses, `GET` vs `HEAD` vs `OPTIONS`, disallowed methods (405), and the feature-off path
- **Security-relevant changes**: traversal (`../`, encoded `%2e%2e`, absolute paths), hidden files, symlinks with `follow_symlinks` on and off, non-ASCII paths
- **Config options**: default value, CLI flag, env var, and TOML key, including TOML overriding CLI
- **Skip**: trivial getters, exact log strings, hyper internals

## Naming and Style

- `fn <feature>_<scenario>_<expectation>()`, e.g. `cache_control_404_uses_no_cache`
- Arrange → act → assert. One scenario per test; several asserts on the same outcome are fine
- Use `assert_eq!` with a message on non-obvious checks: `assert_eq!(v, "br", "pre-compressed variant must win")`
- Loop over methods when behavior must hold for each (`for method in [Method::GET, Method::HEAD]`)

## Checklist

- [ ] A test fails without the change and passes with it
- [ ] Happy path and at least one error path covered
- [ ] Passes under `--features all` and `--no-default-features`
- [ ] New `RequestHandlerOpts` fields are mapped in `testing.rs`
- [ ] New parsers of untrusted input have a proptest
- [ ] Fixtures are minimal and added rather than modified
