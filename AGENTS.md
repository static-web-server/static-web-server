# AGENTS.md — Static Web Server (SWS)

SWS is a cross-platform, high-performance, async static file server written in Rust. Built on `hyper` v1 + `tokio`. Binary target: ~4MB. Edition 2024, MSRV 1.88.0.

## Quick Start

```bash
# Build with all features
cargo build --release --features all

# Format check (exactly as CI runs it)
cargo fmt --all -- --check tests/*.rs

# Lint (must pass with zero warnings before commit), for both feature sets
cargo clippy --features all -- -D warnings
cargo clippy --features all --tests -- -D warnings
cargo clippy --no-default-features -- -D warnings
cargo clippy --no-default-features --tests -- -D warnings

# Run tests
cargo test --features all
cargo test --no-default-features

# Run locally (default port 8080, default root ".")
cargo run --features all -- -d tests/fixtures/public -g trace
```

Pitfalls:
- Use `--features all`, never `--all-features`: the latter enables both `tls-ring` and `tls-fips`, which `src/tls.rs` rejects with `compile_error!`
- `.cargo/config.toml` sets `--cfg tokio_unstable`, but an exported `RUSTFLAGS` replaces it and `--features all` fails with `cannot find function default_runtime_collector`. If the shell sets `RUSTFLAGS`, run `RUSTFLAGS="$RUSTFLAGS --cfg tokio_unstable" cargo ...`
- `--no-default-features` is the strict build: `#![deny(dead_code)]` and `deny(warnings)` turn feature-only items and imports into errors

## Project Rules (non-negotiable)

- `#![forbid(unsafe_code)]` — no `unsafe` anywhere
- `#![deny(missing_docs)]` — every public item, field, and variant needs a `///` doc comment
- No `unwrap()` / `expect()` in production code (use `?` or match)
- No commented-out code — delete it, git preserves history
- All clippy warnings treated as errors
- New dependencies must not increase binary size >100KB unless they replace existing functionality or gate behind a feature flag

## Architecture

```
settings/  →  server/opts.rs  →  service.rs  →  handler.rs  →  static_files/
 (parse)        (init)            (hyper)         (pipeline)      (serve file)
```

### Request Pipeline (linear, 3 phases)

**Pre-processing** (may short-circuit): method check → health → CORS → basic auth → metrics → maintenance → redirects → rewrites → virtual hosts → markdown negotiation

**Core**: `static_files::handle()` — memory cache, path sanitization, resolution (index files, pre-compressed variants), security checks, dir listing, conditional requests (ETag), byte-range

**Post-processing** (additive, runs on every response): fallback page → CORS headers → markdown content-type → text charset → static compression `Vary` → dynamic compression → cache-control → security headers → custom headers

The numbered table with short-circuit behavior lives in `.agents/skills/design/SKILL.md`.

Rules: pre-processing steps return early when they handle the request. Post-processing steps are additive — no step removes headers set by prior steps. Custom headers take final precedence.

### Key Modules

| Module | Purpose |
|--------|---------|
| `settings/` | Parse CLI (clap) / env / TOML, merge, validate |
| `server/` | Bind listener, HTTP/1 or HTTP/2+TLS, graceful shutdown |
| `handler.rs` | Orchestrate request pipeline |
| `static_files/` | `handle()` entry; `resolve.rs`, `security.rs`, `listing.rs`, `reply.rs` |
| `conditional_headers.rs`, `etag.rs` | `If-*` preconditions (304/412), weak `ETag` |
| `compression.rs` | On-the-fly gzip/deflate/brotli/zstd |
| `compression_static.rs` | Serve pre-compressed `.br`/`.gz`/`.zst` files |
| `security_headers.rs` | HSTS, CSP, X-Frame-Options, etc. |
| `control_headers.rs` | Cache-Control based on file extension (2xx/304 only) |
| `custom_headers.rs` | TOML `[[advanced.headers]]` rules, applied last |
| `cors.rs` | CORS preflight and header injection |
| `fs/` | File system: path sanitization, metadata, streaming |
| `directory_listing/` | HTML/JSON directory index |
| `basic_auth.rs` | BCrypt-based HTTP basic auth |
| `error.rs` | Crate error types (`Error`, `Result<T>`) |
| `body.rs` | Unified response body (`BoxBody`) |
| `testing.rs` | `testing::fixtures` helpers for tests |

## Configuration

Three channels, one `General` struct. clap resolves CLI arg → `SERVER_*` env var → compiled default; then, if a config file is loaded (`sws.toml` by default), every key present in its `[general]` table **overrides** that value (`settings/mod.rs`). Advanced rules (`[[advanced.headers]]`, rewrites, redirects, virtual hosts, `[advanced.memory-cache]`) are TOML-only. Feature-gated settings use `#[cfg(feature = "...")]`. Validate everything at startup — never per-request.

Adding an option touches `settings/cli.rs`, `settings/file.rs`, `settings/mod.rs`, `handler.rs` (`RequestHandlerOpts`), `server/opts.rs`, and `testing.rs`; see `.agents/skills/design/SKILL.md`.

## Feature Flags (Cargo + `#[cfg]`)

| Feature | Module | Description |
|---------|--------|-------------|
| `compression` | `compression.rs` + `compression_static.rs` | On-the-fly + pre-compressed serving |
| `directory-listing` | `directory_listing/` | HTML/JSON directory index |
| `directory-listing-download` | (depends on `directory-listing`) | Tar.gz directory download |
| `http2` | (requires `tls`) | HTTP/2 via `hyper-util/http2` |
| `tls` | `tls.rs` | TLS plumbing (requires a crypto provider) |
| `tls-ring` | (default crypto) | TLS via `ring` |
| `tls-fips` | FIPS TLS | TLS via `aws-lc-rs` |
| `basic-auth` | `basic_auth.rs` | BCrypt HTTP basic auth |
| `fallback-page` | `fallback_page.rs` | Custom 404 page |
| `metrics` | `metrics.rs` | Prometheus metrics endpoint |
| `mem-cache` | `mem_cache/` | In-memory file cache (`mini-moka`) |
| `experimental` | `metrics.rs` | Tokio runtime metrics; needs `--cfg tokio_unstable` (set in `.cargo/config.toml`) |

Default features: `compression`, `http2`, `tls-ring`, `directory-listing`, `directory-listing-download`, `basic-auth`, `fallback-page`, `metrics`, `mem-cache`. Aliases: `all` (= `default` + `experimental`), `default-fips`, `all-fips`.

## Coding Conventions

### Error Handling
- Crate types: `crate::Result<T>` and `crate::Error` (anyhow). Use `StatusCode` for HTTP-level errors
- Wrap with context: `fallible_op().with_context(|| "failed to parse config")?`
- Log at boundaries: modules return errors, `handler.rs` logs and converts to status codes

### Async
- Runtime: `tokio` (multi-threaded). HTTP framework: `hyper` v1
- Offload CPU-heavy work via `tokio::task::spawn_blocking`
- Never call `block_on` inside async context

### File System
- Canonicalize root dir once at startup (`server/opts.rs`)
- Path traversal: `sanitize_path()` → `static_files::security::enforce()` (containment → symlink policy → hidden files)
- Prefer `&Path` over `&PathBuf`, `impl AsRef<Path>` for public APIs

### Response
- Use `crate::body::empty()`, `crate::body::full(x)`, or `crate::body::stream(s)`
- Stream files — never buffer full file in memory (exception: small generated responses)

### Performance
- Pre-compute at startup: canonicalize paths, compile regex, build automata
- Avoid allocations in hot path: `&Path` not `PathBuf`, `&[u8]` not `Vec<u8>`
- `#[inline]` on small hot functions, `#[cold]` on error paths
- Prefer `filter_map` over `filter().map()`, `ok_or_else` over `ok_or`

## Testing

### Unit Tests
Location: `#[cfg(test)] mod tests { ... }` at bottom of each source file. Naming: `fn feature_scenario_description()`.

### Integration Tests
Location: `tests/` directory. One file per feature (`tests/compression.rs`, `tests/cors.rs`, etc.). Use fixtures from `tests/fixtures/public/` and TOML configs from `tests/fixtures/toml/`. `fixture_req_handler_opts()` hard-codes `cors`, `page_fallback`, and `memory_cache` to empty; set them explicitly when testing those features.

### Handler Test Pattern
```rust
use static_web_server::testing::fixtures::*;

#[tokio::test]
async fn feature_scenario() {
    let opts = fixture_settings("toml/handler_fixtures.toml");
    let general = General { /* overrides */, ..opts.general };
    let req_handler = fixture_req_handler(fixture_req_handler_opts(general, opts.advanced));
    // Build request, call handle, assert on response
}
```

### Static File Tests
Call `static_files::handle()` directly with `HandleOpts` for isolated file-serving logic tests.

## Security

- Path traversal prevented in layers: `sanitize_path()` → containment (`canonicalize` + prefix check) → symlink component check → hidden file check
- Fail closed: traversal, containment, and hidden-file denials return 404 (not 403) to avoid info leakage; only symlink-policy denials return 403
- Report vulnerabilities privately per `SECURITY.md`
- TLS 1.2+ only. HTTP/2 requires TLS. Security headers auto-enable with TLS
- Never commit secrets, `.env` files, or private keys. TLS keys: `chmod 600`

## Core Principles

- **Correctness above all.** Build production-grade software, not prototypes. Prioritize correctness, reliability, and maintainability over expedient shortcuts.
- **Every change requires a test.** Every code change must be accompanied by a test that fails before the change and passes after it. No behavioral change is complete without objective verification.
- **Enforce invariants explicitly.** Critical assumptions and invariants must be asserted, not silently ignored. Fail fast on invalid states rather than masking defects with defensive conditionals that obscure root causes.
- **Own regressions end-to-end.** Any test failures introduced by your change are your responsibility to investigate and resolve. Do not defer by comparing against another branch or attempting to prove the failure is pre-existing. Diagnose the failure, identify the root cause, and either fix it or provide conclusive evidence that it is unrelated.
- **Evidence over assumptions.** Every debugging hypothesis must be validated with reproducible evidence. Never speculate, infer causality without proof, or implement fixes based on unverified assumptions. Root-cause analysis must be grounded in observable facts.

## Detailed Skill References

For in-depth guidance, load the skill files in `.agents/skills/` (`.claude` is a symlink to `.agents`):

| Skill | Load when |
|-------|-----------|
| `design/SKILL.md` | Adding a config option, pipeline step, feature flag, or cross-module feature. Canonical module map, pipeline table, defaults |
| `rust-backend/SKILL.md` | Editing anything under `src/`; CI-parity commands, lints, feature-gating, module pattern |
| `testing/SKILL.md` | Writing or fixing tests, fixtures, proptests, fuzz targets, benches |
| `security/SKILL.md` | Touching path handling, auth, CORS, TLS, headers, or untrusted input |
| `static-file-serving/SKILL.md` | Compression, caching, ETag, ranges, listing, TOML rules, user-facing examples |
| `performance/SKILL.md` | Hot-path changes, benchmarks, profiling, new dependencies |
| `code-quality/SKILL.md` | Reviewing a diff or checking a change is done |
| `issue-tracking/SKILL.md` | Triaging, reproducing, and fixing bugs |
| `prose/SKILL.md` | Commit messages, CHANGELOG, PR text, rustdoc, docs |

## Further Reading

- Documentation for v3: https://github.com/static-web-server/docs/tree/master/src/v3
- Documentation for v2: https://github.com/static-web-server/docs/tree/master/src/v2
