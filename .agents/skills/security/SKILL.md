---
name: security
description: Review or implement security-sensitive code in the Static Web Server (SWS) project — path traversal defenses, symlink and hidden-file policy, TLS, security headers, CORS, basic auth, untrusted input parsing, and vulnerability handling. Use when touching fs/path.rs, static_files/security.rs, cors.rs, basic_auth.rs, tls.rs, security_headers.rs, redirects/rewrites, directory listing download, or any code that handles request paths or headers.
---

# Security

SWS serves files from disk to untrusted clients. A defect in path handling exposes the host filesystem. Treat every change on the request path as security-relevant.

**When to load**: editing `src/fs/path.rs`, `src/static_files/` (especially `security.rs` and `resolve.rs`), `src/cors.rs`, `src/basic_auth.rs`, `src/tls.rs`, `src/security_headers.rs`, `src/redirects.rs`, `src/rewrites.rs`, `src/markdown.rs`, `src/directory_listing/download.rs`, or reviewing a vulnerability report.

## Principles

- **Fail closed**: if a check cannot complete (I/O error, strip-prefix failure), deny the request
- **Don't leak layout**: traversal, containment, and hidden-file denials return 404, never 403. Only an explicit symlink-policy denial returns 403
- **Defense in depth**: each layer below assumes the previous one may be bypassed
- **No custom crypto**: TLS uses `tokio-rustls` with `ring` (default) or `aws-lc-rs` (FIPS). Passwords use `bcrypt`

## Path Handling Layers

Order as executed in `static_files::handle()`:

### 1. Method allowlist
`HTTP_SUPPORTED_METHODS` (`GET`, `HEAD`, `OPTIONS`) in `exts/http.rs`. Checked in `handler.rs` and again in `static_files::handle()`. Others → 405.

### 2. Sanitization — `fs::path::sanitize_path(base, uri_path)`
Percent-decodes the URI path, then rebuilds it component by component: drops `..`, `.`, root and Windows prefix components, and joins the rest onto `base`. Output is always under `base` lexically.

### 3. Resolution — `static_files::resolve::file_metadata()`
Index files, `.html` suffix fallback, and pre-compressed variant lookup. New candidate paths added here are checked by layer 4 only if they flow into `file_path`; a variant opened separately needs its own check.

### 4. `static_files::security::enforce(file_path, is_dir, opts)`
Runs, in order:
1. **Containment** (`enforce_containment`): `canonicalize()` the probe and require it to start with the canonical base. Escape → 404. Positive results are cached per thread (`thread_local!`, bounded size, wholesale TTL eviction) to skip the syscall; that TTL bounds how long a directory swapped for a symlink at runtime is still trusted
2. **Symlink policy** (`enforce_symlink_policy`, when `--follow-symlinks` is off, the default): `symlink_metadata()` on every component of the relative path. Symlink → 403; check error → 404
3. **Hidden files** (when `--include-hidden` is off, the default): any `Normal` component starting with `.` → 404

Containment is check-then-use: a symlink retargeted between the check and the open can still escape. This is a documented limitation (see `fs::path::is_path_within_base`); do not widen the window by adding I/O between `enforce()` and opening the file.

The base path must already be canonical (root in `server/opts.rs`, virtual-host roots in `settings/`). With `--use-relative-root`, the root is resolved at request time; keep containment semantics intact when touching that path.

### Other path consumers
Every feature that maps a request to a file must reuse these layers, not reimplement them:
- **Markdown negotiation** (`markdown.rs`) checks hidden-file policy and strips the base prefix before rewriting the URI (fixed in #719)
- **Directory download** (`directory_listing/download.rs`) excludes symlinks pointing outside the root from tar.gz archives regardless of `--follow-symlinks`
- **Rewrites/redirects** substitute captured groups into destinations; the rewritten path still goes through layers 2–4. Redirect destinations must not become open redirects to attacker-chosen hosts through placeholder injection
- **Error and fallback pages** are configured paths read at startup, not request-derived

## TLS

- `--tls --tls-cert <pem> --tls-key <pem>`; PKCS#1, PKCS#8, and SEC1 keys are supported (fixtures in `tests/tls/`)
- `tls-ring` and `tls-fips` are mutually exclusive (`compile_error!` in `src/tls.rs`)
- HTTP/2 requires TLS (`http2` feature implies `tls`)
- `--https-redirect` starts a plain-HTTP listener (`--https-redirect-from-port`) that 301-redirects to `--https-redirect-host`; `--https-redirect-from-hosts` (default `localhost`) lists accepted `Host` values; any other host gets 400, which prevents host-header redirect abuse

## Security Headers

`--security-headers` defaults to `true` when `--tls` is set, `false` otherwise. When enabled, `security_headers.rs` inserts on every response:

| Header | Value |
|--------|-------|
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains; preload` |
| `X-Frame-Options` | `DENY` |
| `X-Content-Type-Options` | `nosniff` |
| `Content-Security-Policy` | `frame-ancestors 'self'` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |

HSTS is sent whenever the option is on, including over plain HTTP if the user enables it explicitly. Custom headers run after this step and can override any value.

## CORS

- Off unless `--cors-allow-origins` is set (comma-separated list or `*`)
- Allowed methods are fixed to `GET, HEAD, OPTIONS`
- A request with a disallowed `Origin` gets 403 from `cors::pre_process`
- `--cors-allow-headers`, `--cors-expose-headers` extend the defaults
- Origin parsing is fuzzed (`fuzz/src/cors_origin.rs`) and property-tested; keep both passing when changing it

## Basic Auth

- `--basic-auth "user:$2y$..."`: username, colon, bcrypt hash (`htpasswd -nBC 10 user`). Split on the first `:`
- Runs before metrics, maintenance, redirects, and file serving, so it protects `/metrics` (reordered in #718). `/health` runs before auth by design
- Credentials are base64, not encrypted. Pair with TLS
- No rate limiting; deploy behind a proxy for brute-force protection

## Untrusted Input Checklist

Inputs: URI path and query, `Host`, `Origin`, `Accept`, `Accept-Encoding`, `Range`, `If-*`, `Authorization`, `X-Forwarded-For` / `X-Real-IP` (used for logging only; when `--trusted-proxies` is set, honored only from those peers).

- Parse without panicking: no indexing, `unwrap`, or unchecked arithmetic on attacker-controlled values. Byte-range math (`response/range.rs`) must stay checked or saturating
- Bound allocations by input size (header counts, q-value lists, range sets)
- Add a proptest for each new parser; add a fuzz target for parsers exposed to raw bytes
- Log attacker-controlled strings with `{:?}` so control characters are escaped

## Vulnerability Handling

- Reports arrive privately (see `SECURITY.md`); never discuss details in public issues, commits, or PR titles before the advisory is published
- Fixes land through a GitHub Security Advisory private fork ("Merge commit from fork" in history) and need a v2 LTS backport when v2 is affected
- The fix includes a regression test reproducing the exploit path, and the CHANGELOG entry describes the impact without a ready-made exploit

## Secrets

- Never commit private keys, `.env` files, or configs with real credentials. Test keys in `tests/tls/` are for tests only
- Recommend `chmod 600` for TLS private keys in docs and examples

## Review Checklist

- [ ] New file-path sources pass through `sanitize_path()` and `security::enforce()`
- [ ] Denials return 404 unless the symlink policy is the reason
- [ ] Tests cover `../`, `%2e%2e/`, absolute paths, hidden components, symlinks with the flag on and off, and non-ASCII names
- [ ] No panics reachable from header or path input
- [ ] Auth-protected data is produced after `basic_auth::pre_process`
- [ ] `cargo audit` is clean for new dependencies
