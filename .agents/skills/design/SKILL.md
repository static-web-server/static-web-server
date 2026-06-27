---
name: design
description: Design or review software architecture, API contracts, data models, and module boundaries for the Static Web Server (SWS) project
---

# Software Design

Load this skill when designing new features, refactoring modules, defining configuration interfaces, or reviewing architecture decisions for SWS.

**When to load**: a new feature spans more than one module, a config option is being added or changed, a new step is needed in the request pipeline, a Cargo feature flag is being introduced, or an architecture decision needs review.

## Principles

- **Static file server first**: Every design decision starts from "how does this improve serving static files securely and efficiently?"
- **Small, static binary**: Release binary is ~4MB (uncompressed, musl). New dependencies must not increase it by more than 100KB unless they replace existing functionality or gate behind a feature flag
- **Feature-gate optional functionality**: Every non-core feature lives behind a Cargo feature flag AND a `#[cfg(feature = "...")]` gate. Users compile only what they need
- **Three channels, one source of truth**: CLI args, env vars, and TOML config all map to the same `General` struct. Precedence (highest→lowest): CLI args → env vars → TOML config → compiled defaults
- **Pre-compute at startup, serve at speed**: Resolve, canonicalize, and validate everything possible at server startup. The request hot path is allocation-light and syscall-minimal

## Module Architecture

### Core Pipeline

```
settings/     →  server/opts.rs  →  handler.rs  →  static_files.rs
  (parse)         (init)              (pipeline)      (serve file)
```

### Module Responsibilities

| Module | Responsibility |
|--------|---------------|
| `settings/` | Parse CLI/env/TOML, merge, validate |
| `server/` | Bind listener, start HTTP/1 or HTTP/2+TLS, graceful shutdown |
| `handler.rs` | Orchestrate request pre/post processing pipeline |
| `static_files.rs` | Path resolution, index files, directory listing, byte-range, pre-compressed variants |
| `compression.rs` | On-the-fly gzip/deflate/brotli/zstd compression |
| `compression_static.rs` | Serve pre-compressed `.br`/`.gz`/`.zst` files from disk |
| `security_headers.rs` | Append HSTS, CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy |
| `control_headers.rs` | Append Cache-Control based on file extension |
| `cors.rs` | CORS pre-flight and header injection |
| `fs/` | File system utilities: path sanitization (`fs/path.rs`), metadata (`fs/meta.rs`), streaming (`fs/stream.rs`) |
| `exts/` | HTTP extensions: `Accept-Encoding` parsing, content-coding negotiation (`exts/http.rs`, `exts/headers/`, `exts/mime.rs`) |
| `directory_listing/` | HTML/JSON directory index generation |
| `body.rs` | Unified response body type — `pub type Body = BoxBody<Bytes, io::Error>` (alias, not a struct) |
| `service.rs` | Hyper `Service` bridge: `RouterService` → `RequestService` → `RequestHandler` |

### Feature Flags as Module Boundaries

Every optional feature is both a Cargo feature AND a `#[cfg(feature = "...")]` gate:

- `compression` → `compression.rs` + `compression_static.rs` (meta-feature: `compression-brotli`/`-deflate`/`-gzip`/`-zstd`)
- `directory-listing` → `directory_listing/`
- `http2` → `server/http2.rs` (requires `tls`)
- `tls` (base plumbing, no crypto provider) → `tls.rs` + `server/http1_tls.rs`. `tls-ring` (default) or `tls-fips` selects the provider
- `basic-auth` → `basic_auth.rs`
- `fallback-page` → `fallback_page.rs`
- `metrics` → `metrics.rs`. `experimental` adds `tokio-metrics-collector` and requires `RUSTFLAGS="--cfg tokio_unstable"`
- `mem-cache` → `mem_cache/` (LFU admission + LRU eviction via `mini-moka`, `CompactString` keys)

## Request Pipeline Design

The handler's `handle()` method is the single entry point. The pipeline is linear with three phases:

| # | Step | Phase | Can Short-Circuit? |
|---|------|-------|--------------------|
| 1 | Method check | Pre | Yes (405) |
| 2 | Health/metrics | Pre | Yes |
| 3 | CORS validation | Pre | Yes (preflight 204) |
| 4 | Basic auth | Pre | Yes (401) |
| 5 | Maintenance mode | Pre | Yes (503) |
| 6 | Redirects | Pre | Yes (301/302) |
| 7 | Rewrites | Pre | No (modifies URI, continues) |
| 8 | Virtual hosts | Pre | No (selects config, continues) |
| 9 | Markdown negotiation | Pre | No (sets content-type hint, continues) |
| 10 | `static_files::handle()` | Core | No (always produces a response) |
| 11 | Fallback page | Post | No |
| 12 | CORS headers | Post | No |
| 13 | Markdown content-type | Post | No |
| 14 | Text charset | Post | No |
| 15 | Static compression vary | Post | No |
| 16 | Dynamic compression | Post | No |
| 17 | Cache-Control | Post | No |
| 18 | Security headers | Post | No |
| 19 | Custom headers | Post | No |

When adding a new step, specify its position relative to an existing step by number.

### Design Rules for the Pipeline

1. **Pre-processing steps return early** when they handle the request (CORS preflight, redirect, health check)
2. **Post-processing steps are additive** — they append or modify headers. No step removes headers set by prior steps
3. **Order matters**: static compression runs before dynamic compression (pre-compressed files avoid CPU cost), cache-control runs before security headers (custom headers — applied last — take final precedence)

## Configuration Design

### The `General` struct

CLI arguments, environment variables, and TOML keys all resolve to the same `General` struct:

```
--port 8787  ↔  SERVER_PORT=8787  ↔  [general] port = 8787
```

### TOML Config File

Advanced features (custom headers, rewrites, redirects, virtual hosts) are TOML-only. They live in a separate `Advanced` struct and require glob/regex patterns.

### Default Values Philosophy

- **Secure by default**: Hidden files ignored, symlinks disabled, security headers enabled when TLS is active
- **Performant by default**: Compression, cache-control, and HTTP/2 are default-on Cargo features (opt-out via `--no-default-features`). Disable them to reduce binary size if not needed
- **Conservative where it matters**: Grace period at 0 (explicit opt-in), directory listing off, CORS off

## File Serving Design

### Index File Resolution

1. Request for `/` or `/dir/` → directory detected via metadata
2. Try each index file in the user-configured order (default: `["index.html", "index.htm"]`)
3. For each index candidate: check pre-compressed variant, then regular file, then `.html` suffix fallback
4. If no index found → directory listing (if enabled) or 404

### Pre-compressed Variant Priority

Static (on-disk) compression is tried before dynamic (on-the-fly). The client's `Accept-Encoding` header determines variant selection. SWS honors quality values.

### Byte-Range Serving

`static_files.rs` supports `Range: bytes=` for partial content delivery. Multi-range responses use `multipart/byteranges`.

## Review Checklist

- [ ] Is the new code behind an appropriate feature flag if optional?
- [ ] Does the request pipeline order make sense (pre → core → post)?
- [ ] Are paths canonicalized once, not per-request?
- [ ] Can every public function be tested with the existing fixture infrastructure?
- [ ] Are error states mapped to appropriate HTTP status codes?
- [ ] Does the config change work across all three channels (CLI, env, TOML)?
