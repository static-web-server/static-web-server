---
name: rust-conventions
description: Write or modify Rust code in the Static Web Server (SWS) project — modules, types, error handling, async code, hyper responses, feature-gated code, and the local commands that mirror CI. Use when editing anything under src/, adding a module or dependency, or getting clippy/fmt/test/rustdoc to pass.
---

# Rust Conventions

Architecture, pipeline order, and the config-option recipe: `design/SKILL.md`. Naming, function shape, comments, log wording: `coding-style/SKILL.md`. Review criteria: `code-quality/SKILL.md`.

**When to load**: editing any file under `src/`, adding a module or dependency, changing error handling or async code, or fixing a lint, fmt, test, or docs failure.

## Verify Like CI (`.github/workflows/devel.yml`)

```bash
cargo fmt --all -- --check tests/*.rs

cargo clippy --features all -- -D warnings
cargo clippy --features all --tests -- -D warnings
cargo clippy --no-default-features -- -D warnings
cargo clippy --no-default-features --tests -- -D warnings

cargo test --features all
cargo test --no-default-features
```

FIPS (`--no-default-features --features all-fips`) needs `cmake`, `golang`, `libclang`; run it only when touching TLS provider code.

Rustdoc (nightly) catches broken intra-doc links and missing docs:

```bash
cargo +nightly rustdoc --lib -Zrustdoc-map --features all \
    --config "build.rustflags=[\"--cfg\", \"tokio_unstable\"]" \
    -Zhost-config -Ztarget-applies-to-host \
    --config "host.rustflags=[\"--cfg\", \"tokio_unstable\"]" \
    --config "build.rustdocflags=[\"--cfg\", \"docsrs\", \"--cfg\", \"tokio_unstable\", \"-Z\", \"unstable-options\", \"--cap-lints\", \"warn\", \"--extern-html-root-takes-precedence\"]" \
    -Zunstable-options -- --document-private-items
```

### Pitfalls

- **Never `--all-features`**: it enables `tls-ring` and `tls-fips` together and `src/tls.rs` fails with `compile_error!`. Use `--features all` (`make lint` still uses `--all-features`; don't rely on it)
- **`RUSTFLAGS` replaces `.cargo/config.toml`**, which sets `--cfg tokio_unstable`. An exported `RUSTFLAGS` makes `--features all` fail with `cannot find function default_runtime_collector`. Fix: `RUSTFLAGS="$RUSTFLAGS --cfg tokio_unstable" cargo ...` (forces a full rebuild)
- **`--no-default-features` is the strict build**: unused imports, dead code, and unused variables that exist only under a feature surface here. Run it whenever you add `#[cfg(feature = ...)]`

## Crate-Level Lints (`src/lib.rs`)

```rust
#![deny(missing_docs)]       // every pub item, field, variant needs `///`
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]
```

Integration test files repeat `forbid(unsafe_code)`, `deny(warnings)`, `deny(rust_2018_idioms)`, `deny(dead_code)`. Every source file starts with the SPDX header:

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>
```

## Error Handling

- **Types**: `crate::Result<T>` (`anyhow::Result`), `crate::Error` (`anyhow::Error`); `src/error.rs` re-exports `Context`, `anyhow!`, `bail!`
- **HTTP-level failures**: file-serving functions return `Result<T, StatusCode>`; `static_files::handle()` returns `Result<StaticFileResponse, StatusCode>`; `handler.rs` turns a status into an error page via `error_page::error_response()`
- **Startup errors**: `fallible().with_context(|| format!("unable to read {}", path.display()))?`; abort with a readable message
- **No `unwrap()` / `expect()` in production code**: `?`, `ok_or_else`, `let ... else`, or match. Tests may unwrap
- **Log at the boundary**: modules return errors; `handler.rs` and `error_page.rs` log them with `tracing` macros, never `println!`
- **Avoid**: `String` as an error type, `Box<dyn Error>`, discarding a `Result`

## Async and HTTP

- `tokio` multi-threaded; never `block_on` in async context; `spawn_blocking` for CPU-heavy or blocking work
- `hyper` v1 + `hyper-util` + `http-body-util`; `service.rs` adapts `RequestHandler` to `hyper::service::Service`
- `crate::body::Body` is `BoxBody<Bytes, io::Error>`: `body::empty()`, `body::full(bytes)`, `body::stream(s)`. Stream files; buffer only small generated responses
- Static header values: `static NAME: HeaderValue = HeaderValue::from_static("...")`, then `.clone()`
- Method helpers: `MethodExt` (`is_allowed`, `is_get`, `is_head`, `is_options`) from `crate::exts::http`

## Feature Module Pattern

```rust
/// Initializes <feature>.
pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.my_feature = enabled;
    tracing::info!(enabled, "my feature");
}

/// Pre-processing: `Some` short-circuits the request.
pub(crate) fn pre_process<T>(opts: &RequestHandlerOpts, req: &Request<T>)
    -> Option<Result<Response<Body>, Error>> { /* ... */ }

/// Post-processing: always returns the (possibly modified) response.
pub(crate) fn post_process<T>(opts: &RequestHandlerOpts, req: &Request<T>, resp: Response<Body>)
    -> Result<Response<Body>, Error> { /* ... */ }
```

Keep the testable logic in a plain function (`append_headers(resp, hsts)`, `auto(method, headers, level, resp)`) that unit tests and `benches/` call directly.

## Feature-Gated Code

- Gate the module in `lib.rs` with `#[cfg(feature = "x")]` and `#[cfg_attr(docsrs, doc(cfg(feature = "x")))]`
- Gate every field, constructor entry, and use site. `RequestHandlerOpts` is built in `handler.rs` (`Default`), `server/opts.rs`, and `testing.rs`
- Compression gates on `any(feature = "compression", feature = "compression-gzip", feature = "compression-brotli", feature = "compression-zstd", feature = "compression-deflate")`; copy the existing block
- Provide a `#[cfg(not(feature = "x"))]` fallback value when a caller needs one (`will_serve_fallback` in `handler.rs`); placement rules in `coding-style/SKILL.md`

## File System

- Root canonicalized once in `server/opts.rs` (unless `--use-relative-root`); virtual-host roots in `settings/`. Never canonicalize the base per request
- Request paths: `fs::path::sanitize_path()` then `static_files::security::enforce()`. Read `security/SKILL.md` before touching either
- `fs/meta.rs` (`try_metadata`, `try_file_open`, `try_metadata_with_html_suffix`, `try_markdown_variant`) maps metadata or open failures to `StatusCode::NOT_FOUND` and logs at `debug`/`trace`
- Prefer `&Path` over `&PathBuf`, `impl AsRef<Path>` for new public APIs

## Code Style

Naming, function shape, nesting, comments, `#[cfg]` placement, and log wording live in `coding-style/SKILL.md`; load it with this skill. Mechanics that belong here:

- `rustfmt` defaults. Derive `Debug` and `Clone` on config types; `#[must_use]` on pure functions
- No global mutable state. Shared config is `Arc<RequestHandlerOpts>`; per-thread caches use `thread_local!`; process-wide registries use `LazyLock` (metrics)
- `regex-lite`, not `regex`; `globset` for globs; `aho-corasick` for placeholder replacement

## Dependencies

- Binary-size budget (100KB) and how to measure it: `performance/SKILL.md`, Binary Size
- `default-features = false`, enable only what's needed
- `cargo audit` runs in CI (`.github/workflows/audit.yml`)
