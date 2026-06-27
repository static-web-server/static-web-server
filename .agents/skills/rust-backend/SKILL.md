---
name: rust-backend
description: Write or review Rust backend code for the Static Web Server (SWS) project — crates, modules, functions, types, error handling, and async code
---

# Rust Backend Coding Standards

Load this skill when writing, reviewing, or refactoring Rust code in the SWS project.

**When to load**: editing any file under `src/`, adding a new module, changing error handling, touching async code, or reviewing a PR that modifies Rust source.

## Mandatory Tools

**Always** use the following commands to maintain code quality and consistency:

### Linting

1. `cargo clippy --features all -- -D warnings`
2. `cargo clippy --features all --tests -- -D warnings`

### Formatting

1. `cargo fmt --all -- --check tests/*.rs`

### Testing

1. `cargo test -v --features all`
2. `cargo test -v --no-default-features`

### Cargo docs lint

```
cargo +nightly rustdoc --lib -Zrustdoc-map --features all \
    --config "build.rustflags=[\"--cfg\", \"tokio_unstable\"]" \
    -Zhost-config -Ztarget-applies-to-host \
    --config "host.rustflags=[\"--cfg\", \"tokio_unstable\"]" \
    --config "build.rustdocflags=[\"--cfg\", \"docsrs\", \"--cfg\", \"docsrs\", \"--cfg\", \"tokio_unstable\", \"-Z\", \"unstable-options\", \"--emit=invocation-specific\", \"--cap-lints\", \"warn\", \"--extern-html-root-takes-precedence\"]" \
    -Zunstable-options -- --document-private-items
```

## Code Quality

- **All clippy commands in Mandatory Tools must pass with zero warnings before committing**
- **`unsafe` is forbidden at the crate level**: SWS uses `#![forbid(unsafe_code)]`. Consider refactoring to avoid `unsafe` entirely
- **Prefer `&Path` over `&PathBuf`** in function parameters. Accept `impl AsRef<Path>` for public APIs
- **Use `#[must_use]`** on pure functions whose return value should not be silently discarded
- **Derive common traits explicitly**: `Debug`, `Clone`, `PartialEq`, `Eq` on all public types unless there is a reason not to
- **No commented-out code**: Delete it. Git history preserves it

## Error Handling

- **Use the crate's `Result<T>` and `Error` types**: Defined in `src/error.rs`. All fallible functions return `Result<T>` or `Result<T, StatusCode>` for HTTP-level errors
- **Use `anyhow::Context` for wrapping**: `fallible_op().with_context(|| "failed to parse config")?`
- **No `unwrap()` or `expect()` in production code**: Use `?` or match. Allow `expect` only for values guaranteed by prior validation (e.g., a regex that is known to compile, a lock that should never be poisoned). Add an inline comment explaining the invariant
- **Log errors at the boundary**: Module code returns errors. The HTTP handler (`handler.rs`) logs them and converts to HTTP status codes
- **Distinguish HTTP status codes from internal errors**: `StatusCode` (hyper) for HTTP semantics; `Error` (anyhow) for internal failures. Functions use `Result<T, StatusCode>` when the only possible failures are HTTP-level

## Async Code

- **Use `tokio` as the runtime**: All async code targets `tokio` (multi-threaded, `rt-multi-thread` feature)
- **No `block_on` in async context**: Never call `tokio::runtime::Handle::block_on` inside an async function
- **Prefer `spawn_blocking` for CPU-bound work**: Offload file hashing, compression dictionary building, etc.
- **Use `hyper` as the HTTP framework**: SWS is built on `hyper` v1 with `http-body-util`. `src/service.rs` defines `RouterService` and `RequestService`, which implement `hyper::service::Service` and delegate to `RequestHandler::handle()`

## HTTP & Request Handling

- **Request pipeline ordering**: `handler.rs` orchestrates the request flow in a fixed order with three phases:
  - **Pre-processing** (may short-circuit with a response): method check → health/metrics → CORS → basic auth → maintenance mode → redirects → rewrites → virtual hosts → markdown negotiation
  - **Core**: static file resolution and serving
  - **Post-processing** (additive, runs on every response): fallback page → CORS headers → text charset → static compression → dynamic compression → cache-control → security headers → custom headers
- **Post-processing is additive**: Each post-processing step appends or modifies headers. No step removes headers set by a previous step unless explicitly documented
- **Response body type**: `crate::body::Body` is a type alias for `BoxBody<Bytes, std::io::Error>` (defined in `src/body.rs`). Use the constructors `crate::body::empty()`, `crate::body::full(impl Into<Bytes>)`, or `crate::body::stream<S>(s)`
- **Static file serving is the core**: `static_files.rs` handles path resolution, index files, directory listing, pre-compressed variants, and byte-range requests

## Settings & Configuration

- **Three equivalent channels**: CLI arguments (`clap`), environment variables, and TOML config file. Defined in `src/settings/`
- **Precedence (lowest to highest)**: compiled defaults → TOML config file → environment variables → CLI arguments. Runtime validation runs after merging
- **Feature-gated settings**: Settings that require Cargo features (e.g., `compression`, `directory-listing`) are conditionally compiled with `#[cfg(feature = "...")]`
- **Validation at startup, not per-request**: Canonicalize paths, validate TLS certificates, parse index files once at startup in `server/opts.rs`

## File System

- **Canonicalize paths once at startup**: The root directory is canonicalized in `server/opts.rs`. Per-request path resolution reuses this canonical base
- **Path traversal prevention**: `sanitize_path()` (in `src/fs/path.rs`) strips `..`, root prefixes, and other traversal components. `resolve_and_contain()` (in `src/static_files/security.rs`) verifies the resolved path stays within the base directory
- **Symlink policy**: When `--follow-symlinks` is disabled (default), `enforce_symlink_policy()` (in `src/static_files/security.rs`) walks each path component checking for symlinks via `symlink_metadata()`. This is a syscall per component — check cheaper guards (hidden files) first
- **File metadata operations**: `try_metadata()` and `try_metadata_with_html_suffix()` (in `src/fs/meta.rs`) encapsulate filesystem access with proper error mapping to HTTP status codes

## Patterns to Avoid

- **No `String` as an error type**: Use structured errors or `StatusCode`
- **No `Box<dyn Error>`**: Use `anyhow::Error`
- **No global mutable state**: No `static mut` or `lazy_static!` with `Mutex`. SWS uses `Arc<RequestHandlerOpts>` for shared read-only config
- **No deep nesting**: Extract nested conditionals into named functions or match guards
- **No per-request canonicalize or alloc when avoidable**: Cache canonical paths, reuse buffers, avoid `clone()` in the hot path
