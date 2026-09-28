---
name: design
description: Design or review architecture for the Static Web Server (SWS) project — module boundaries, the request pipeline, configuration options, Cargo feature flags, and defaults. Use when adding a feature that spans modules, adding or changing a config option, inserting a pipeline step, introducing a feature flag, or reviewing an architecture decision.
---

# Software Design

Canonical reference for SWS architecture: module map, pipeline order, configuration model, feature flags, defaults. Other skills link here instead of repeating it. Values below are taken from the source; re-check the cited file before relying on one.

**When to load**: a change spans modules, a config option is added or changed, a pipeline step is added or moved, a Cargo feature is introduced, or a design proposal needs review.

## Principles

- **Static file server first**: a feature belongs in SWS only if it improves serving static files securely and efficiently
- **Small binary**: a new dependency must not grow the release binary by more than 100KB unless it replaces existing functionality or sits behind a feature flag
- **Feature-gate optional functionality**: a Cargo feature plus `#[cfg(feature = "...")]` in code
- **Pre-compute at startup**: canonicalize paths, compile regex/globs, build automata, validate config once in `settings/` and `server/opts.rs`. The request path reads `Arc<RequestHandlerOpts>` and never re-validates
- **Secure defaults**: hidden files and symlinks refused, CORS and directory listing off, proxy headers trusted from nobody, HSTS only over TLS

## Module Map

```
bin/server.rs → settings/ → server/ (mod.rs, opts.rs) → service.rs → handler.rs → static_files/
                (parse+merge)  (bind, init handler opts)  (hyper Service)  (pipeline)   (serve file)
```

| Module | Responsibility |
|--------|---------------|
| `settings/cli.rs` | `General`: clap CLI args + `SERVER_*` env vars; `///` docs are the `--help` text |
| `settings/file.rs` | TOML `Settings`: `General` (all `Option<T>`) + `Advanced` (headers, rewrites, redirects, virtual hosts, memory cache). Unknown keys reported via `serde_ignored` |
| `settings/mod.rs` | Merge CLI/env with TOML, compile advanced rules (globs → regex, Aho-Corasick), build `Settings { general, advanced }` |
| `server/opts.rs` | Build `RequestHandlerOpts`: canonicalize root, call each feature's `init()` |
| `server/` | `listener.rs` (TCP, `--fd`, Unix socket), `http1.rs`, `http1_tls.rs`, `http2.rs`, `uds.rs` accept loops; `redirect.rs` (HTTP→HTTPS server); `idle_timeout.rs` (connection IO wrapper); `conn_limit.rs` (`--max-connections` slots); `browser.rs` (`--open`); graceful shutdown in `mod.rs` |
| `service.rs` | `RouterService` → `RequestService` → `RequestHandler::handle()` |
| `handler.rs` | `RequestHandlerOpts` and the request pipeline |
| `static_files/` | `mod.rs::handle()` entry; `resolve.rs` (metadata, index files, pre-compressed variant), `security.rs` (containment, symlink, hidden), `listing.rs`, `reply.rs`, `opts.rs` (`HandleOpts`, `StaticFileResponse`) |
| `fs/` | `path.rs` (`sanitize_path`), `meta.rs` (metadata → `StatusCode`), `stream.rs` (file streaming, buffer sizing) |
| `response/` | Byte ranges (`range.rs`) and file response building |
| `conditional_headers.rs`, `etag.rs` | `If-*` preconditions (304/412), weak `ETag` |
| `compression.rs`, `compression_static.rs` | On-the-fly encoding; pre-compressed `.br` `.gz` `.zst` selection |
| `control_headers.rs`, `security_headers.rs`, `custom_headers.rs`, `text_charset.rs` | Post-processing header steps |
| `health.rs`, `cors.rs`, `basic_auth.rs`, `metrics.rs`, `maintenance_mode.rs`, `redirects.rs`, `rewrites.rs`, `virtual_hosts.rs`, `markdown.rs` | Pre-processing steps |
| `error_page.rs`, `fallback_page.rs` | 404/50x pages, SPA fallback |
| `exts/` | `http.rs` (`MethodExt`, `HTTP_SUPPORTED_METHODS`), `headers/` (`Accept-Encoding`, q-values, `ContentCoding`), `mime.rs` (compressible detection) |
| `mem_cache/` | In-memory file cache (`mini-moka`), keyed by `(path, ContentCoding)` |
| `directory_listing/` | HTML/JSON index, sorting, tar.gz download |
| `access_logs.rs`, `log_addr.rs`, `logger.rs` | Per-request access log, trusted-proxy address logging, tracing setup |
| `body.rs` | `Body = BoxBody<Bytes, io::Error>` with `empty()`, `full()`, `stream()` |
| `error.rs` | `Result<T>`, `Error` (anyhow); re-exports `Context`, `anyhow!`, `bail!` |
| `testing.rs` | `testing::fixtures` helpers (`#[doc(hidden)]`) |
| `https_redirect.rs`, `tls.rs`, `winservice.rs`, `signals.rs` | HTTPS redirect handler, TLS config, Windows service, signals |

## Request Pipeline

`RequestHandler::handle()` in `src/handler.rs`. Order as implemented:

| # | Step | Short-circuits? |
|---|------|-----------------|
| 1 | Method check (`GET`, `HEAD`, `OPTIONS`) | 405 |
| 2 | Health probes `/livez`, `/readyz` (`--health`) | 200, or 503 from `/readyz` during shutdown or when the root is missing |
| 3 | CORS (`cors::pre_process`) | Preflight reply, or 403 for a disallowed origin |
| 4 | Basic auth | 401. Applies to `OPTIONS` too, except a CORS preflight (CORS on, `Origin` + `Access-Control-Request-Method`) |
| 5 | Metrics endpoint (after auth, so `/metrics` is protected) | Yes |
| 6 | Maintenance mode | 503 or configured status |
| 7 | Redirects | 301/302 |
| 8 | Rewrites | Yes when the rule has `redirect`; else rewrites the URI and continues |
| 9 | Virtual hosts | No (swaps `base_path` by `Host`) |
| 10 | Markdown negotiation (`--accept-markdown`) | No (swaps the URI path to the `.md` variant) |
| 11 | `static_files::handle()`: mem-cache lookup (`GET`/`HEAD`), sanitize, resolve, security, `OPTIONS`/trailing-slash/listing, conditional + range reply | Errors become error pages |
| 12 | Fallback page | — |
| 13 | CORS response headers | — |
| 14 | Markdown: `Vary: Accept` (HTML, markdown, listings) and `Content-Type` for the variant | — |
| 15 | Text charset (`charset=utf-8` on `text/*`) | — |
| 16 | Static compression `Vary` | — |
| 17 | Dynamic compression | — |
| 18 | `Cache-Control` | — |
| 19 | Security headers | — |
| 20 | Custom headers (final precedence). Steps 1–8 return *early* responses; only `scope = "all"` rules reach them | — |

Around the pipeline: metrics (request count, inflight, duration, bytes; health probes excluded) and `access_logs::AccessLog`, which wraps the body and logs one line per request when the body is dropped, with the status and the request values the client sent (before rewrites).

### Pipeline Rules

1. Pre-processing: `module::pre_process(&opts, req) -> Option<Result<Response<Body>, Error>>`; `Some` short-circuits
2. Post-processing: `module::post_process(&opts, req, resp) -> Result<Response<Body>, Error>`; always returns a response
3. Post-processing is additive: a step may overwrite a header it owns, never remove another step's. Custom headers run last
4. Pre-compressed variants are chosen inside `static_files`, so dynamic compression skips bodies that already have `Content-Encoding`
5. A new step is placed by number and justified against auth (step 4): anything that exposes data runs after it

## Configuration Model

```
--port 8080  ↔  SERVER_PORT=8080  ↔  [general] port = 8080
```

**Precedence** (`settings/mod.rs`): clap resolves CLI arg → `SERVER_*` env → compiled default; then every key present in the loaded file's `[general]` table **overwrites** it. Advanced options (`[[advanced.headers]]`, `[[advanced.rewrites]]`, `[[advanced.redirects]]`, `[[advanced.virtual-hosts]]`, `[advanced.memory-cache]`) are TOML-only.

TOML keys are kebab-case. Default file `./sws.toml` (`--config-file` / `-w`); `./config.toml` still loads with a deprecation warning.

### Adding a Config Option

1. `settings/cli.rs`: field on `General` with `#[arg(long, default_value = ..., env = "SERVER_...")]` and a `///` doc. Booleans use `default_missing_value("true"), num_args(0..=1), action = ArgAction::Set` so `--flag`, `--flag true`, `--flag=false` all work
2. `settings/file.rs`: `Option<T>` field on file `General` (or `Advanced`)
3. `settings/mod.rs`: `let mut x = opts.x;`, the `if let Some(v) = general.x { x = v }` merge, the field in the final `General { .. }`
4. `handler.rs`: field on `RequestHandlerOpts` and its `Default`, if the request path needs it
5. `server/opts.rs`: `module::init(value, &mut handler_opts)`, which logs the effective value with `tracing::info!`
6. `testing.rs`: map it in `fixture_req_handler_opts()` or the crate fails to compile
7. Tests: parsing in `tests/settings.rs` or a TOML fixture under `tests/fixtures/toml/`, plus behavior tests
8. Docs: user docs live in `static-web-server/docs` (`src/v3/`); note the follow-up in the PR. `CHANGELOG.md` is written by the maintainer

Validate in `settings/` or `server/opts.rs`; fail startup with `bail!`/`with_context`. Never validate per request.

### Defaults (`settings/cli.rs`)

| Option | Default |
|--------|---------|
| `--host` / `--port` | `::` / `8080` |
| `--root` | `.` |
| `--index-files` | `index.html` |
| `--log-level` (`-g`) / `--log-format` | `error` / `json` (`pretty` for development) |
| `--compression` / `--compression-level` / `--compression-static` | `true` / `default` / `true` |
| `--cache-control-headers` / `--etag` / `--text-charset` | `true` |
| `--redirect-trailing-slash` | `true` |
| `--security-headers` | `true` when built with `tls`, else `false`. HSTS only with `--tls` |
| `--directory-listing` | `false` (format `html`; `auto` negotiates via `Accept`; order `6` = unsorted) |
| `--include-hidden` / `--follow-symlinks` | `false` |
| `--cors-allow-origins` | empty (CORS off) |
| `--health` / `--metrics` / `--accept-markdown` / `--open` / `--maintenance-mode` | `false` |
| `--log-remote-address` / `--log-x-real-ip` / `--log-forwarded-for` | `false`; proxy headers only from `--trusted-proxies` (default: none) |
| `--access-log-format` | empty (structured fields) |
| `--log-trace-context` | `false`; requires `--log-format json` |
| `--threads-multiplier` / `--max-blocking-threads` | `1` / `512` (`2` / `20` on wasm) |
| `--grace-period` / `--idle-timeout` / `--max-header-size` | `0` / `30` s (`0` disables) / `16384` bytes (431 when exceeded) |
| `--page404` / `--page50x` / `--page-fallback` | `./404.html` / `./50x.html` / unset |
| `--max-connections` | `0` (no limit) |
| Memory cache | off unless `[advanced.memory-cache]` exists |

## Feature Flags

| Feature | Gates | Notes |
|---------|-------|-------|
| `compression` | `compression.rs` | Meta-feature for `compression-{brotli,deflate,gzip,zstd}`; code gates on `any(...)` of all five |
| `http2` | `server/http2.rs` | Implies `tls` |
| `tls` | `tls.rs`, `server/http1_tls.rs`, HTTPS redirect | No crypto provider by itself |
| `tls-ring` / `tls-fips` | provider | Mutually exclusive: `src/tls.rs` `compile_error!`s if both or neither |
| `directory-listing` | `directory_listing/` | |
| `directory-listing-download` | tar.gz download | Implies `directory-listing`, `compression-gzip` |
| `basic-auth`, `fallback-page`, `metrics`, `mem-cache` | the module of the same name | |
| `experimental` | tokio runtime metrics | Needs `--cfg tokio_unstable` (`.cargo/config.toml`) |

Aliases: `default`, `all` (= `default` + `experimental`), `default-fips`, `all-fips`. Use `--features all`, never `--all-features`.

CI builds `--no-default-features`, `--features all`, and `--no-default-features --features all-fips`; all must compile with zero warnings. `#![deny(dead_code)]` makes an item used under one feature an error under the others. Gating mechanics: `rust-conventions/SKILL.md`, Feature-Gated Code.

## Review Checklist

- [ ] Optional functionality is feature-gated and builds under all three CI feature sets
- [ ] New pipeline steps are placed by number and respect the auth boundary
- [ ] Post-processing remains additive; custom headers still apply last, including `scope = "all"` on early responses
- [ ] Paths, regexes, globs prepared at startup, not per request
- [ ] New config options cover CLI, env, TOML, `testing.rs`, and tests
- [ ] Error states map to status codes without leaking layout (404 over 403 for traversal)
