---
name: rust-backend
description: Write or modify Rust code in the Static Web Server (SWS) project — modules, types, error handling, async code, hyper responses, feature-gated code, and the local commands that mirror CI. Use when editing anything under src/, adding a module or dependency, or getting clippy/fmt/test/rustdoc to pass.
---

# Rust Backend Coding Standards

Load this skill before editing Rust source. For architecture, pipeline order, and the config-option recipe, see `design/SKILL.md`. For review criteria, see `code-quality/SKILL.md`.

**When to load**: editing any file under `src/`, adding a module or dependency, changing error handling or async code, or fixing a CI failure in lint, fmt, tests, or docs.

## Verify Like CI

Run these before declaring a change done. They match `.github/workflows/devel.yml`.

```bash
# Format (CI runs exactly this)
cargo fmt --all -- --check tests/*.rs

# Clippy: lib/bin, then tests. Repeat for each feature set CI checks
cargo clippy --features all -- -D warnings
cargo clippy --features all --tests -- -D warnings
cargo clippy --no-default-features -- -D warnings
cargo clippy --no-default-features --tests -- -D warnings

# Tests
cargo test --features all
cargo test --no-default-features

# Rustdoc (nightly) — catches broken intra-doc links and missing docs
cargo +nightly rustdoc --lib -Zrustdoc-map --features all \
    --config "build.rustflags=[\"--cfg\", \"tokio_unstable\"]" \
    -Zhost-config -Ztarget-applies-to-host \
    --config "host.rustflags=[\"--cfg\", \"tokio_unstable\"]" \
    --config "build.rustdocflags=[\"--cfg\", \"docsrs\", \"--cfg\", \"docsrs\", \"--cfg\", \"tokio_unstable\", \"-Z\", \"unstable-options\", \"--cap-lints\", \"warn\", \"--extern-html-root-takes-precedence\"]" \
    -Zunstable-options -- --document-private-items
```

FIPS (`--no-default-features --features all-fips`) needs `cmake`, `golang`, and `libclang`; run it only when touching TLS provider code.

### Tooling Pitfalls

- **Never use `--all-features`**: it enables `tls-ring` and `tls-fips` together and `src/tls.rs` fails with `compile_error!`. Use `--features all`. (`make lint` still uses `--all-features`; do not rely on it)
- **`RUSTFLAGS` replaces `.cargo/config.toml` flags**: the config sets `--cfg tokio_unstable`. If the shell exports `RUSTFLAGS` (e.g. `-L native=...` for musl), `--features all` fails with `cannot find function default_runtime_collector in crate tokio_metrics_collector`. Fix: `RUSTFLAGS="$RUSTFLAGS --cfg tokio_unstable" cargo ...`. Changing `RUSTFLAGS` also forces a full rebuild
- **`--no-default-features` is the strict build**: unused imports, dead code, and unused variables that only exist under a feature surface here. Run it whenever you add `#[cfg(feature = ...)]`

## Crate-Level Lints

`src/lib.rs` sets:

```rust
#![deny(missing_docs)]       // every pub item, field, and variant needs a `///` doc comment
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]
```

Integration test files under `tests/` repeat `forbid(unsafe_code)`, `deny(warnings)`, `deny(rust_2018_idioms)`, and `deny(dead_code)`.

Source files begin with the SPDX header:

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>
```

## Error Handling

- **Types**: `crate::Result<T>` (`anyhow::Result`) and `crate::Error` (`anyhow::Error`) from `src/error.rs`, which also re-exports `Context`, `anyhow!`, and `bail!`
- **HTTP-level failures**: file-serving functions return `Result<T, StatusCode>`. `static_files::handle()` returns `Result<StaticFileResponse, StatusCode>`; `handler.rs` turns the status into an error page via `error_page::error_response()`
- **Context on startup errors**: `fallible().with_context(|| format!("unable to read {}", path.display()))?`. Startup errors abort with a readable message; request-path errors become status codes
- **No `unwrap()` / `expect()` in production code**: use `?`, `ok_or_else`, or a match. Tests may unwrap
- **Log at the boundary**: modules return errors; `handler.rs` and `error_page.rs` log them. Use `tracing` macros (`error!`, `warn!`, `debug!`, `trace!`), not `println!`
- **Avoid**: `String` as an error type, `Box<dyn Error>`, silently discarding a `Result`

## Async and HTTP

- **Runtime**: `tokio` multi-threaded. Never call `block_on` in async context. Use `tokio::task::spawn_blocking` for CPU-heavy or blocking work
- **HTTP**: `hyper` v1 + `hyper-util` + `http-body-util`. `service.rs` adapts `RequestHandler` to `hyper::service::Service`
- **Bodies**: `crate::body::Body` is `BoxBody<Bytes, io::Error>`. Build with `body::empty()`, `body::full(bytes)`, or `body::stream(s)`. Stream files; buffer only small generated responses (health, error pages, listings)
- **Static header values**: define `static NAME: HeaderValue = HeaderValue::from_static("...")` and `.clone()` it (see `control_headers.rs`, `security_headers.rs`)
- **Method helpers**: use `MethodExt` (`is_allowed`, `is_head`, `is_options`) from `crate::exts::http`

## Feature Module Pattern

Each feature module follows the same shape so `server/opts.rs` and `handler.rs` stay uniform:

```rust
/// Initializes <feature>.
pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.my_feature = enabled;
    tracing::info!(enabled, "my feature");
}

/// Pre-processing: return `Some` to short-circuit the request.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> { /* ... */ }

/// Post-processing: always return the (possibly modified) response.
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    resp: Response<Body>,
) -> Result<Response<Body>, Error> { /* ... */ }
```

Keep the testable logic in a plain function (e.g. `append_headers(uri, &mut resp)`, `auto(method, headers, level, resp)`) that unit tests and `benches/` call directly.

## Feature-Gated Code

- Gate the module in `lib.rs` with `#[cfg(feature = "x")]` and `#[cfg_attr(docsrs, doc(cfg(feature = "x")))]`
- Gate every field, constructor entry, and use site. `RequestHandlerOpts` is built in three places: its `Default` impl (`handler.rs`), `server/opts.rs`, and `testing.rs`
- Compression code gates on `any(feature = "compression", feature = "compression-gzip", feature = "compression-brotli", feature = "compression-zstd", feature = "compression-deflate")` — copy the existing block
- Provide a `#[cfg(not(feature = "x"))]` fallback value when a caller needs one (see `will_serve_fallback` in `handler.rs`)

## File System

- The root is canonicalized once in `server/opts.rs` (unless `--use-relative-root`); virtual-host roots are canonicalized in `settings/`. Never canonicalize the base per request
- Request paths pass through `fs::path::sanitize_path()` then `static_files::security::enforce()`. See `security/SKILL.md` before touching either
- `fs/meta.rs` (`try_metadata`, `try_file_open`, `try_metadata_with_html_suffix`) maps any metadata or open failure to `StatusCode::NOT_FOUND` and logs the cause at `debug`/`trace`
- Prefer `&Path` over `&PathBuf` in new signatures, and `impl AsRef<Path>` for new public APIs

## Code Style

- `rustfmt` defaults. No commented-out code; git keeps history
- Derive `Debug` and `Clone` on config/option types; `#[must_use]` on pure functions whose result must be used
- No global mutable state (`static mut`, `Mutex` in a `static`). Shared config is `Arc<RequestHandlerOpts>`; per-thread caches use `thread_local!` (see the containment cache in `static_files/security.rs`)
- Regex: the crate uses `regex-lite`, not `regex`. Globs use `globset`. Placeholder replacement uses `aho-corasick`
- Extract nested conditionals into named functions; prefer `let ... else` and `if let ... &&` chains (edition 2024) over deep nesting

## Dependencies

- Check binary-size impact before adding a crate: `cargo build --release --features all` before/after, compare `target/release/static-web-server`. Over 100KB requires a feature flag or replacing existing functionality
- Use `default-features = false` and enable only needed features
- `cargo audit` runs in CI (`.github/workflows/audit.yml`)
