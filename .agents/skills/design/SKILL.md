---
name: design
description: Design or review architecture for the Static Web Server (SWS) project — module boundaries, the request pipeline, configuration options, Cargo feature flags, and defaults. Use when adding a feature that spans modules, adding or changing a config option, inserting a pipeline step, introducing a feature flag, or reviewing an architecture decision.
---

# Software Design

This skill is the canonical reference for SWS architecture: the module map, the request pipeline order, the configuration model, and feature flags. Other skills link here instead of repeating it.

**When to load**: a change spans more than one module, a config option is added or changed, a pipeline step is added or moved, a Cargo feature is introduced, or a design proposal needs review.

## Principles

- **Static file server first**: a feature belongs in SWS only if it improves serving static files securely and efficiently
- **Small binary**: a new dependency must not grow the release binary by more than 100KB unless it replaces existing functionality or sits behind a feature flag
- **Feature-gate optional functionality**: a non-core feature is a Cargo feature in `Cargo.toml` plus `#[cfg(feature = "...")]` gates in code
- **Pre-compute at startup**: canonicalize paths, compile regex/globs, build automata, and validate config once in `settings/` and `server/opts.rs`. The request path reads from `Arc<RequestHandlerOpts>` and never re-validates
- **Secure defaults**: hidden files and symlinks are refused, CORS and directory listing are off

## Module Map

```
bin/server.rs → settings/ → server/ (mod.rs, opts.rs) → service.rs → handler.rs → static_files/
                 (parse+merge)  (bind, init handler opts)   (hyper Service)  (pipeline)   (serve file)
```

| Module | Responsibility |
|--------|---------------|
| `settings/cli.rs` | `General` struct: clap CLI args + `SERVER_*` env vars |
| `settings/file.rs` | TOML `Settings`: `General` (all `Option<T>`) + `Advanced` (headers, rewrites, redirects, virtual hosts, memory cache). Unknown keys reported via `serde_ignored` |
| `settings/mod.rs` | Merge CLI/env with TOML, compile advanced rules (globs, regex, Aho-Corasick), build `Settings { general, advanced }` |
| `server/opts.rs` | Build `RequestHandlerOpts` by calling each feature's `init()` |
| `server/` | Listeners (TCP, `--fd`, Unix socket `uds.rs`), HTTP/1, HTTP/1+TLS, HTTP/2, HTTPS redirect server, browser launch (`browser.rs`), graceful shutdown |
| `service.rs` | `RouterService` → `RequestService` → `RequestHandler::handle()` |
| `handler.rs` | `RequestHandlerOpts` and the request pipeline |
| `static_files/` | `mod.rs::handle()` entry; `resolve.rs` (metadata, index files, pre-compressed variant), `security.rs` (containment, symlink, hidden checks), `listing.rs`, `reply.rs` |
| `fs/` | `path.rs` (`sanitize_path`, `PathExt`), `meta.rs` (metadata → `StatusCode`), `stream.rs` (file streaming, buffer sizing) |
| `response/` | Byte-range (`range.rs`) and file response building |
| `conditional_headers.rs`, `etag.rs` | `If-*` preconditions (304/412) and weak `ETag` |
| `compression.rs` / `compression_static.rs` | On-the-fly encoding / pre-compressed `.br` `.gz` `.zst` selection |
| `control_headers.rs`, `security_headers.rs`, `custom_headers.rs`, `text_charset.rs` | Post-processing header steps |
| `cors.rs`, `basic_auth.rs`, `health.rs`, `metrics.rs`, `maintenance_mode.rs`, `redirects.rs`, `rewrites.rs`, `virtual_hosts.rs`, `markdown.rs` | Pre-processing steps |
| `error_page.rs`, `fallback_page.rs` | 404/50x pages, SPA fallback |
| `exts/` | `http.rs` (`MethodExt`, `HTTP_SUPPORTED_METHODS`), `headers/` (`Accept-Encoding`, q-values), `mime.rs` (text/compressible detection) |
| `mem_cache/` | In-memory file cache (`mini-moka`) |
| `directory_listing/` | HTML/JSON index, sorting, tar.gz download |
| `body.rs` | `pub type Body = BoxBody<Bytes, io::Error>` with `empty()`, `full()`, `stream()` |
| `error.rs` | `Result<T> = anyhow::Result<T>`, `Error = anyhow::Error`, re-exports `Context`, `anyhow!`, `bail!` |
| `testing.rs` | `testing::fixtures` helpers for tests (`#[doc(hidden)]`) |
| `winservice.rs`, `signals.rs`, `logger.rs`, `log_addr.rs` | Platform/service integration and logging |

## Request Pipeline

`RequestHandler::handle()` in `src/handler.rs` is the single entry point. Order as implemented:

| # | Step | Phase | Short-circuits? |
|---|------|-------|-----------------|
| 0 | Remote address logging (`log_addr`) | Pre | No |
| 1 | Method check (`GET`, `HEAD`, `OPTIONS` only) | Pre | Yes (405) |
| 2 | Health endpoint (`health`) | Pre | Yes |
| 3 | CORS (`cors::pre_process`) | Pre | Yes (preflight reply, or 403 for a disallowed origin) |
| 4 | Basic auth | Pre | Yes (401) |
| 5 | Metrics endpoint (after auth, so `/metrics` is protected) | Pre | Yes |
| 6 | Maintenance mode | Pre | Yes (503 or configured status) |
| 7 | Redirects | Pre | Yes (301/302) |
| 8 | Rewrites | Pre | Yes when the rule has `redirect`; otherwise rewrites the URI and continues |
| 9 | Virtual hosts | Pre | No (swaps `base_path` by `Host`) |
| 10 | Markdown negotiation (`--accept-markdown`) | Pre | No (swaps the URI path to the `.md` variant) |
| 11 | `static_files::handle()` (mem-cache lookup, sanitize, resolve, security, listing, conditional/range reply) | Core | Errors become error pages |
| 12 | Fallback page | Post | — |
| 13 | CORS response headers | Post | — |
| 14 | Markdown `Content-Type` | Post | — |
| 15 | Text charset (`charset=utf-8` on `text/*`) | Post | — |
| 16 | Static compression `Vary` | Post | — |
| 17 | Dynamic compression | Post | — |
| 18 | `Cache-Control` | Post | — |
| 19 | Security headers | Post | — |
| 20 | Custom headers (final precedence) | Post | — |

Metrics (request count, inflight, duration) wrap the whole pipeline.

### Pipeline Rules

1. Pre-processing steps return `Some(result)` to short-circuit, `None` to continue. Follow the existing `module::pre_process(&self.opts, req) -> Option<Result<Response<Body>, Error>>` shape
2. Post-processing steps take and return `Response<Body>`: `module::post_process(&self.opts, req, resp) -> Result<Response<Body>, Error>`
3. Post-processing is additive. A step may overwrite a header it owns; it does not remove headers set by earlier steps. Custom headers run last so users can override anything
4. Pre-compressed variants are chosen inside `static_files` before dynamic compression runs, so dynamic compression skips already-encoded bodies
5. When proposing a new step, name its position by the numbers above and justify it against auth (steps 4–5): anything that exposes data must run after basic auth

## Configuration Model

One option reaches the handler through three channels:

```
--port 8080  ↔  SERVER_PORT=8080  ↔  [general] port = 8080
```

**Precedence** (verified in `settings/mod.rs::parse_from`):

1. clap resolves CLI arg → `SERVER_*` env var → compiled default into `cli::General`
2. If a config file is loaded, every key present in its `[general]` table **overwrites** the value from step 1
3. Advanced options (`[[advanced.headers]]`, `[[advanced.rewrites]]`, `[[advanced.redirects]]`, `[[advanced.virtual-hosts]]`, `[advanced.memory-cache]`) exist only in TOML

TOML keys are kebab-case (`#[serde(rename_all = "kebab-case")]`). `./config.toml` is still read with a deprecation warning; the default name is `sws.toml`.

### Adding a Config Option (touch points)

1. `src/settings/cli.rs` — field on `General` with `#[arg(long, default_value = ..., env = "SERVER_...")]` and a `///` doc comment (it becomes `--help` text; `#![deny(missing_docs)]` is on). Booleans follow the existing `default_missing_value("true"), num_args(0..=1), action = ArgAction::Set` pattern so `--flag`, `--flag true`, and `--flag=false` all work
2. `src/settings/file.rs` — `Option<T>` field on file `General` (or `Advanced`)
3. `src/settings/mod.rs` — `let mut x = opts.x;`, the `if let Some(v) = general.x { x = v }` merge, and the field in the final `General { .. }`
4. `src/handler.rs` — field on `RequestHandlerOpts` and its `Default` impl, if the request path needs it
5. `src/server/opts.rs` — set it on `handler_opts`, usually via `module::init(value, &mut handler_opts)` which also logs the effective value with `tracing::info!`
6. `src/testing.rs` — map it in `fixture_req_handler_opts()` or the crate fails to compile
7. Tests — CLI/env/TOML parsing in `tests/settings.rs` or a TOML fixture under `tests/fixtures/toml/`, plus behavior tests
8. Docs — user docs live in the separate `static-web-server/docs` repo (`src/v3/`); note the follow-up in the PR. Add a `CHANGELOG.md` entry

Validate in `settings/` or `server/opts.rs` and fail startup with context (`bail!`/`with_context`) on invalid input. Never validate per request.

### Defaults (from `settings/cli.rs`)

| Option | Default |
|--------|---------|
| `--host` / `--port` | `::` / `8080` |
| `--root` | `.` |
| `--index-files` | `index.html` |
| `--log-level` (`-g`) | `error` |
| `--compression` / `--compression-level` | `true` / `default` |
| `--compression-static` | `true` |
| `--cache-control-headers` | `true` |
| `--etag` | `true` |
| `--text-charset` | `true` |
| `--redirect-trailing-slash` | `true` |
| `--security-headers` | `false`, `true` when `--tls` is set |
| `--directory-listing` | `false` (format `html`; `auto` negotiates via `Accept`) |
| `--include-hidden` / `--follow-symlinks` | `false` / `false` |
| `--cors-allow-origins` | empty (CORS off) |
| `--health` / `--metrics` / `--accept-markdown` / `--open` | `false` |
| `--threads-multiplier` / `--max-blocking-threads` | `1` / `512` (`2` / `20` on wasm) |
| `--grace-period` | `0` |
| Memory cache | off unless `[advanced.memory-cache]` exists |

## Feature Flags

| Feature | Gates | Notes |
|---------|-------|-------|
| `compression` | `compression.rs` | Meta-feature for `compression-{brotli,deflate,gzip,zstd}`; code gates on `any(feature = "compression", feature = "compression-gzip", ...)` |
| `http2` | `server/http2.rs` | Implies `tls` |
| `tls` | `tls.rs`, `server/http1_tls.rs`, `https_redirect` | No crypto provider by itself |
| `tls-ring` / `tls-fips` | provider selection | Mutually exclusive: `src/tls.rs` emits `compile_error!` if both or neither are on |
| `directory-listing` | `directory_listing/` | |
| `directory-listing-download` | tar.gz download | Implies `directory-listing`, `compression-gzip` |
| `basic-auth` | `basic_auth.rs` | |
| `fallback-page` | `fallback_page.rs` | |
| `metrics` | `metrics.rs` | Prometheus |
| `mem-cache` | `mem_cache/` | |
| `experimental` | tokio runtime metrics | Needs `--cfg tokio_unstable` (set in `.cargo/config.toml`) |

Aliases: `default`, `all` (= `default` + `experimental`), `default-fips`, `all-fips`. Use `--features all`, never `--all-features` (it enables both TLS providers).

Feature design rules:
- Every feature combination CI builds (`--no-default-features`, `--features all`, `--no-default-features --features all-fips`) must compile with zero warnings. `#![deny(dead_code)]` turns an item used only under one feature into an error under the others; gate the item, not only its callers
- Struct fields that exist only under a feature carry the `#[cfg]` on the field, every constructor (`Default`, `testing.rs`, `server/opts.rs`), and every use site
- Add `#[cfg_attr(docsrs, doc(cfg(feature = "...")))]` to feature-gated public items

## Review Checklist

- [ ] Optional functionality sits behind a feature flag and builds under all three CI feature sets
- [ ] New pipeline steps are placed by number and respect the auth boundary
- [ ] Post-processing remains additive; custom headers still apply last
- [ ] Paths, regexes, and globs are prepared at startup, not per request
- [ ] New config options cover CLI, env, TOML, `testing.rs`, and tests
- [ ] Error states map to HTTP status codes without leaking filesystem layout (404 over 403 for traversal)
