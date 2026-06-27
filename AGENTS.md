# AGENTS.md — Static Web Server (SWS)

SWS is a cross-platform, high-performance, async static file server written in Rust. Built on `hyper` v1 + `tokio`. Binary target: ~4MB. Edition 2024, MSRV 1.88.0.

## Quick Start

```bash
# Build with all features
cargo build --release --features all

# Lint (must pass with zero warnings before commit)
cargo clippy --features all -- -D warnings
cargo clippy --features all --tests -- -D warnings

# Format check
cargo fmt --all -- --check

# Run tests
cargo test -v --features all
cargo test -v --no-default-features
```

## Project Rules (non-negotiable)

- `#![forbid(unsafe_code)]` — no `unsafe` anywhere
- No `unwrap()` / `expect()` in production code (use `?` or match)
- No commented-out code — delete it, git preserves history
- All clippy warnings treated as errors
- New dependencies must not increase binary size >100KB unless they replace existing functionality or gate behind a feature flag

## Architecture

```
settings/  →  server/opts.rs  →  handler.rs  →  static_files.rs
 (parse)        (init)             (pipeline)      (serve file)
```

### Request Pipeline (linear, 3 phases)

**Pre-processing** (may short-circuit): method check → health/metrics → CORS → basic auth → maintenance → redirects → rewrites → virtual hosts → markdown negotiation

**Core**: `static_files::handle()` — path resolution, index files, dir listing, pre-compressed variants, byte-range

**Post-processing** (additive, runs on every response): fallback page → CORS headers → text charset → static compression → dynamic compression → cache-control → security headers → custom headers

Rules: pre-processing steps return early when they handle the request. Post-processing steps are additive — no step removes headers set by prior steps. Custom headers take final precedence.

### Key Modules

| Module | Purpose |
|--------|---------|
| `settings/` | Parse CLI (clap) / env / TOML, merge, validate |
| `server/` | Bind listener, HTTP/1 or HTTP/2+TLS, graceful shutdown |
| `handler.rs` | Orchestrate request pipeline |
| `static_files.rs` | Path resolution, index files, dir listing, byte-range, pre-compressed |
| `compression.rs` | On-the-fly gzip/deflate/brotli/zstd |
| `compression_static.rs` | Serve pre-compressed `.br`/`.gz`/`.zst` files |
| `security_headers.rs` | HSTS, CSP, X-Frame-Options, etc. |
| `control_headers.rs` | Cache-Control based on file extension |
| `cors.rs` | CORS preflight and header injection |
| `fs/` | File system: path sanitization, metadata, streaming |
| `directory_listing/` | HTML/JSON directory index |
| `basic_auth.rs` | BCrypt-based HTTP basic auth |
| `error.rs` | Crate error types (`Error`, `Result<T>`) |
| `body.rs` | Unified response body (`BoxBody`) |

## Configuration

Three equivalent channels, one `General` struct. Precedence (highest→lowest): CLI args → env vars → TOML config → compiled defaults. Feature-gated settings use `#[cfg(feature = "...")]`. Validate everything at startup — never per-request.

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
| `experimental` | `metrics.rs` + `mem_cache/` | Requires `RUSTFLAGS="--cfg tokio_unstable"` |

Default features: `compression`, `http2`, `tls-ring`, `directory-listing`, `directory-listing-download`, `basic-auth`, `fallback-page`, `metrics`, `mem-cache`.

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
- Path traversal: `sanitize_path()` → `resolve_and_contain()` → symlink policy check → hidden file check
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
Location: `tests/` directory. One file per feature (`tests/compression.rs`, `tests/cors.rs`, etc.). Use fixtures from `tests/fixtures/public/`.

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

- Path traversal prevented at 4 layers: `sanitize_path()` → `resolve_and_contain()` → symlink component check → hidden file check
- Fail closed: traversal attempts return 404 (not 403) to avoid info leakage
- TLS 1.2+ only. HTTP/2 requires TLS. Security headers auto-enable with TLS
- Never commit secrets, `.env` files, or private keys. TLS keys: `chmod 600`

## Core Principles

- **Correctness above all.** Build production-grade software, not prototypes. Prioritize correctness, reliability, and maintainability over expedient shortcuts.
- **Every change requires a test.** Every code change must be accompanied by a test that fails before the change and passes after it. No behavioral change is complete without objective verification.
- **Enforce invariants explicitly.** Critical assumptions and invariants must be asserted, not silently ignored. Fail fast on invalid states rather than masking defects with defensive conditionals that obscure root causes.
- **Own regressions end-to-end.** Any test failures introduced by your change are your responsibility to investigate and resolve. Do not defer by comparing against another branch or attempting to prove the failure is pre-existing. Diagnose the failure, identify the root cause, and either fix it or provide conclusive evidence that it is unrelated.
- **Evidence over assumptions.** Every debugging hypothesis must be validated with reproducible evidence. Never speculate, infer causality without proof, or implement fixes based on unverified assumptions. Root-cause analysis must be grounded in observable facts.

## Detailed Skill References

For in-depth guidance, see the skill files in `.agents/skills/`:

| Skill | File |
|-------|------|
| Software Design | `.agents/skills/design/SKILL.md` |
| Rust Backend | `.agents/skills/rust-backend/SKILL.md` |
| Testing | `.agents/skills/testing/SKILL.md` |
| Security | `.agents/skills/security/SKILL.md` |
| Static File Serving | `.agents/skills/static-file-serving/SKILL.md` |
| Performance | `.agents/skills/performance/SKILL.md` |
| Prose/Writing | `.agents/skills/prose/SKILL.md` |
| Issue Tracking | `.agents/skills/issue-tracking/SKILL.md` |

## Further Reading

- Documentation for v3: https://github.com/static-web-server/docs/tree/master/src/v3
- Documentation for v2: https://github.com/static-web-server/docs/tree/master/src/v2
