---
name: security
description: Review or implement security-sensitive code in the Static Web Server (SWS) project — path traversal defenses, symlink and hidden-file policy, TLS, security headers, CORS, basic auth, proxy headers, connection limits, untrusted input parsing, and vulnerability handling. Use when touching fs/path.rs, static_files/security.rs, cors.rs, basic_auth.rs, tls.rs, security_headers.rs, access_logs.rs, log_addr.rs, redirects/rewrites, directory listing download, or any code that handles request paths or headers.
---

# Security

SWS serves files from disk to untrusted clients. Every change on the request path is security-relevant.

**When to load**: editing `src/fs/path.rs`, `src/static_files/` (especially `security.rs`, `resolve.rs`), `src/cors.rs`, `src/basic_auth.rs`, `src/tls.rs`, `src/security_headers.rs`, `src/access_logs.rs`, `src/log_addr.rs`, `src/redirects.rs`, `src/rewrites.rs`, `src/markdown.rs`, `src/directory_listing/download.rs`, `src/mem_cache/`, or handling a vulnerability report.

## Principles

- **Fail closed**: if a check cannot complete (I/O error, strip-prefix failure), deny
- **Don't leak layout**: traversal, containment, and hidden-file denials return 404. Only an explicit symlink-policy denial returns 403
- **Defense in depth**: each layer assumes the previous one may be bypassed
- **Auth before data**: anything that returns file content or metadata runs after `basic_auth::pre_process`, for every method
- **No custom crypto**: TLS via `tokio-rustls` with `ring` (default) or `aws-lc-rs` (FIPS); passwords via `bcrypt`

## Path Handling Layers

Order as executed in `static_files::handle()`:

1. **Method allowlist**: `GET`, `HEAD`, `OPTIONS` (`exts/http.rs`), checked in `handler.rs` and again in `static_files`. Others → 405
2. **Sanitization**: `fs::path::sanitize_path(base, uri_path)` percent-decodes, then rebuilds component by component, dropping `..`, `.`, root and Windows prefixes. Output is lexically under `base`
3. **Memory cache lookup** (`GET`/`HEAD` only): a hit serves content that already passed layers 4–5 when it was inserted
4. **Resolution**: `resolve::file_metadata()` picks index files, `.html` suffix fallback, pre-compressed variants
5. **`security::enforce(file_path, is_dir, opts)`**, run on the file actually served (index file, `.html` sibling, and the pre-compressed variant separately):
   1. Containment: `canonicalize()` the path, require the canonical base as prefix. Escape → 404. Positive results are cached per thread (bounded size, wholesale TTL eviction); the TTL bounds how long a directory swapped for a symlink stays trusted
   2. Symlink policy (when `--follow-symlinks` is off): `symlink_metadata()` on every component. Symlink → 403; check error → 404
   3. Hidden files (when `--include-hidden` is off): any component starting with `.` → 404. So is an 8.3 short-name alias (`GIT~1` for `.git` on NTFS and FAT): a component containing `~` with no same-name entry, ignoring case, in its parent directory. Only `~` components cost a directory read; an unlistable parent → 404

Containment is check-then-use: a symlink retargeted between check and open can still escape (documented in `fs::path::is_path_within_base`). Don't widen the window by adding I/O between `enforce()` and opening the file.

The base must already be canonical (root in `server/opts.rs`, virtual-host roots in `settings/`). With `--use-relative-root` the root is resolved at request time; keep containment intact there.

### Other path consumers

Every feature that maps a request to a file reuses these layers:
- **Markdown negotiation** applies the hidden-file policy (dotfiles on the request, short-name aliases on the variant found) and strips the base before rewriting the URI
- **Directory download** excludes symlinks pointing outside the root from archives regardless of `--follow-symlinks`
- **Rewrites/redirects** substitute captures into destinations; the rewritten path still goes through layers 2–5. Redirect destinations must not become open redirects through placeholder injection
- **Error and fallback pages** are configured paths read at startup, not request-derived

## Basic Auth

- `--basic-auth "user:$2y$..."`: username, colon, bcrypt hash (`htpasswd -nBC 10 user`). Split on the first `:`
- Runs before metrics, maintenance, redirects, and file serving, so `/metrics` is protected. `/livez` and `/readyz` run before auth by design
- Applies to every method including `OPTIONS`. The only exception is a CORS preflight (CORS enabled, `Origin` and `Access-Control-Request-Method` present), which browsers send without credentials
- Credentials are base64, not encrypted: pair with TLS. No rate limiting: deploy behind a proxy for brute-force protection

## CORS

- Off unless `--cors-allow-origins` is set (comma-separated list or `*`)
- Allowed methods are fixed to `GET, HEAD, OPTIONS`; a disallowed `Origin` gets 403 from `cors::pre_process`
- `--cors-allow-headers`, `--cors-expose-headers` extend the defaults; invalid names are dropped with a log line instead of panicking
- Origin parsing is fuzzed (`fuzz/src/cors_origin.rs`) and property-tested

## TLS

- `--tls --tls-cert <pem> --tls-key <pem>`; PKCS#1, PKCS#8, SEC1 keys (fixtures in `tests/tls/`). TLS 1.2+
- `tls-ring` and `tls-fips` are mutually exclusive; HTTP/2 requires TLS
- `--https-redirect` starts a plain-HTTP listener (`--https-redirect-from-port`) that 301-redirects to `--https-redirect-host`; `--https-redirect-from-hosts` (default `localhost`) lists accepted `Host` values, others get 400

## Security Headers

`--security-headers` defaults to `true` in builds with the `tls` feature, `false` otherwise. When enabled, every response gets:

| Header | Value |
|--------|-------|
| `X-Frame-Options` | `DENY` |
| `X-Content-Type-Options` | `nosniff` |
| `Content-Security-Policy` | `frame-ancestors 'self'` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains`, **only with `--tls`** (RFC 6797 forbids it over plain HTTP) |

Behind a TLS-terminating proxy, HSTS is added at the proxy or with an `[[advanced.headers]]` rule. Custom headers run after this step and can override any value.

## Connection and Request Limits

- `--idle-timeout` (default 30 s): closes a connection with no read or write progress; an HTTP/1 request header must also arrive within it. `0` disables
- `--max-header-size` (default 16384 bytes): request line + headers for HTTP/1, header list for HTTP/2. Exceeded → 431
- `--max-connections` (default `0`, no limit): connections served at the same time, shared by all listeners (`server/conn_limit.rs`). A slot is taken after `accept`, so an idle listener never holds one; once full, new connections wait in the listen backlog. It relies on the idle timeout to free slots of connections that send nothing
- Accept-loop errors back off instead of spinning
- Metrics labels are bounded: method is `GET`/`HEAD`/`OPTIONS` or `other`, status is its class (`2xx`), host is a configured virtual host or `other`

## Untrusted Input

Inputs: URI path and query, `Host`, `Origin`, `Accept`, `Accept-Encoding`, `Range`, `If-*`, `Authorization`, `Sec-Fetch-Dest`, `User-Agent`, `Referer`, `X-Forwarded-For` / `X-Real-IP`.

- Proxy headers are logged only for requests from `--trusted-proxies` (empty by default: trust nobody). IPv4 entries also match `::ffff:a.b.c.d`
- Parse without panicking: no indexing, `unwrap`, or unchecked arithmetic on attacker-controlled values. Byte-range math (`response/range.rs`) stays checked or saturating
- Bound allocations by input size (header counts, q-value lists, range sets)
- Access log values from the request are escaped (`\xHH` for non-printable, `"`, `\`) and cut at 4 KiB, so a client cannot inject lines or terminal escapes. Elsewhere, log attacker-controlled strings with `{:?}`
- Add a proptest for each new parser; add a fuzz target for parsers exposed to raw bytes

## Vulnerability Handling

- Reports arrive privately (`SECURITY.md`); no details in public issues, commits, or PR titles before publication
- Fixes land through a GitHub Security Advisory private fork ("Merge commit from fork" in history); v2 LTS (`2.x`) gets a backport when affected. The fork has no CI
- Draft the advisory with `security-advisory/SKILL.md`; review the fork PR with `pr-review/SKILL.md` (Advisory PR)
- The fix includes a regression test that drives the exploit through the full pipeline

## Secrets

Never commit private keys, `.env` files, or configs with real credentials. Test keys in `tests/tls/` are for tests only. Recommend `chmod 600` for TLS keys in docs.

## Review Checklist

- [ ] New file-path sources pass through `sanitize_path()` and `security::enforce()` on the file actually served
- [ ] Denials return 404 unless the symlink policy is the reason
- [ ] Tests cover `../`, `%2e%2e/`, absolute paths, hidden components, symlinks with the flag on and off, non-ASCII names
- [ ] No panics reachable from header or path input
- [ ] Data-returning code runs after `basic_auth::pre_process`, for `OPTIONS` too
- [ ] New per-request labels, keys, or log fields are bounded
- [ ] `cargo audit` is clean for new dependencies
