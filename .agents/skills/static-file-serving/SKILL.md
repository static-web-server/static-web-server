---
name: static-file-serving
description: Explain or change how the Static Web Server (SWS) serves files — index resolution, MIME types, pre-compressed and on-the-fly compression, Cache-Control, ETag and conditional requests, byte ranges, directory listing, SPA fallback, markdown negotiation, and TOML headers/rewrites/redirects/virtual hosts. Use when working on static_files/, compression, control_headers, custom_headers, directory_listing, or writing user-facing config examples.
---

# Static File Serving

How SWS turns a request path into a response, and how users configure it. All values below are taken from the source; re-check the cited file before changing a default.

**When to load**: changing file resolution, compression, caching, conditional or range handling, directory listing, or custom headers; answering a "how do I serve X with SWS" question; writing CLI or TOML examples.

## Resolution (`static_files/`)

1. `sanitize_path()` maps the URI path under the root (see `security/SKILL.md`)
2. Memory cache lookup (`mem-cache`, only when `[advanced.memory-cache]` is configured)
3. `resolve::file_metadata()`:
   - Directory → try each `--index-files` entry in order (default: `index.html`)
   - Missing file → try `<path>.html` (so `/about` serves `about.html`)
   - With `--compression-static`, look for a pre-compressed sibling
4. `security::enforce()` (containment, symlinks, hidden files)
5. Directory without trailing slash → 308 redirect when `--redirect-trailing-slash` (default `true`)
6. `OPTIONS` → `204 No Content` with `Allow` and `Accept-Ranges`
7. Directory listing if enabled and no index file
8. File reply: conditional checks, byte range, streamed body

## MIME Types and Charset

- `Content-Type` comes from `mime_guess` on the file extension. There is no MIME override flag; use a custom header rule
- `--text-charset` (default `true`) appends `charset=utf-8` to `text/*` responses that lack a charset (`text_charset.rs`)
- `exts/mime.rs` decides which types are text-like (`text/*`, `+json`/`+xml` suffixes, and a fixed list of `application/*` types) and therefore compressible

## Compression

### Pre-compressed (`--compression-static`, default `true`)

For `file.ext`, SWS looks for `file.ext.br`, `file.ext.gz`, or `file.ext.zst`, walking the client's `Accept-Encoding` in q-value order. `gzip` and `deflate` both map to `.gz`. The variant is served with `Content-Encoding` and `Vary: Accept-Encoding`, and dynamic compression then skips it.

```bash
brotli -q 11 -k dist/app.js      # app.js.br
gzip -9 -k dist/app.js           # app.js.gz
zstd -19 -k dist/app.js          # app.js.zst
```

### On-the-fly (`--compression`, default `true`)

`compression.rs` encodes the response when all hold:
- method is not `HEAD` or `OPTIONS`
- the response has no `Content-Encoding` yet
- the client accepts an encoding compiled in (`deflate`, `gzip`, `br`, `zstd`, depending on features)
- the MIME type is compressible
- `Content-Length` is absent or at least **200 bytes** (`MIN_COMPRESS_SIZE`)

`--compression-level` is `fastest`, `default`, or `best`. Defaults per algorithm: gzip/deflate/brotli 4, zstd 3.

## Cache-Control (`--cache-control-headers`, default `true`)

`control_headers.rs` sets `Cache-Control` by the URI extension (case-insensitive), **only for `2xx` and `304`**. Every other status gets `no-cache` so a missing asset is not cached for a year.

| Extensions | Value |
|-----------|-------|
| `avif bmp bz2 css doc gif gz htc ico jpeg jpg js jxl map mjs mp3 mp4 ogg ogv pdf png rar rtf tar tgz wav weba webm webp woff woff2 zip` | `max-age=31536000` |
| `atom rss` | `max-age=3600` |
| anything else (including `html`, `json`, `xml`, `svg`, no extension) | `no-cache` |

Both arrays must stay sorted: lookup uses `binary_search`. Override per path with a custom header rule.

## ETag and Conditional Requests (`--etag`, default `true`)

- Weak validator from metadata: `W/"<mtime_ns_hex>-<len_hex>"` (`etag.rs`)
- `conditional_headers.rs` evaluates `If-Match` / `If-Unmodified-Since` (412), then `If-None-Match` / `If-Modified-Since` (304). `If-None-Match` takes precedence over `If-Modified-Since`
- `Last-Modified` is sent when the filesystem reports a modification time
- `If-Range` is honored for byte ranges

## Byte Ranges (`response/range.rs`)

`Range: bytes=` requests get `206 Partial Content` with `Content-Range`; unsatisfiable ranges get 416.

## Directory Listing (`--directory-listing`, default `false`)

- `--directory-listing-format html|json|auto`; `auto` picks via `Accept` and adds `Vary: Accept`
- `--directory-listing-order 0..5` (name/modified/size, asc/desc); `6` (default) is unordered. Clients can override per request with `?sort=N`
- `--directory-listing-download targz` adds a tar.gz download link (`directory-listing-download` feature). Archives exclude symlinks that point outside the root

## Fallback and Error Pages

- `--page-fallback <file>` (`fallback-page` feature): served with 200 for `GET` requests that would 404, for SPA client-side routing. The path is **not** relative to the root
- `--page404` (default `./404.html`) and `--page50x` (default `./50x.html`): HTML bodies for error responses, loaded at startup. Relative paths resolve under the root; a missing file falls back to a generic message

## Markdown Negotiation (`--accept-markdown`, default `false`)

When the request `Accept` header lists `text/markdown`, SWS looks for `<path>.md`, then `<path>.html.md`, then `<path>/index.html.md`, serves it as `text/markdown`, and applies the hidden-file policy to the variant.

## Advanced TOML Rules

Advanced rules are TOML-only, use glob `source` patterns matched against the URI path, and are arrays of tables (`[[...]]`):

```toml
[advanced]

# Custom headers: applied last, override any earlier header
[[advanced.headers]]
source = "/assets/**"
status = [200, 206, 304]          # optional; omit to apply to every status
headers = { Cache-Control = "public, max-age=31536000, immutable" }

[[advanced.headers]]
source = "**/*.html"
headers = { Cache-Control = "no-cache" }

# Redirects: short-circuit with 301 or 302
[[advanced.redirects]]
source = "/pages/{*}.html"
destination = "/?p=$1"
kind = 301

# Rewrites: change the path internally; add `redirect = 301` to redirect instead
[[advanced.rewrites]]
source = "/error-page/{404,50x}.html"
destination = "/$1.html"

# Virtual hosts: pick a root by the Host header
[[advanced.virtual-hosts]]
host = "example.com"
root = "./sites/example"

# In-memory cache (mem-cache feature); values are clamped to maximums
[advanced.memory-cache]
capacity = 100        # entries (max 100000)
ttl = 1800            # seconds (max 86400)
tti = 300             # seconds idle (max 3600)
max-file-size = 8192  # KiB (max 32768)
```

Globs compile through `globset` with `literal_separator(true)` and are then rewritten into capturing regex groups (`settings/mod.rs`): `{*}` matches within one segment, `{**}` spans segments, `{a,b}` is alternation. `$1`, `$2`, ... reference captures in pattern order; in `**/error-page.{html}` the `{html}` group is `$2`. See `tests/fixtures/toml/rewrites.toml` and `redirects.toml` for working rules.

## Common Setups

```bash
# SPA with pre-compressed assets
static-web-server -d ./dist --page-fallback ./dist/index.html

# Production TLS (security headers turn on automatically)
static-web-server -d ./dist --tls --tls-cert cert.pem --tls-key key.pem --http2

# Local file browsing
static-web-server -d . --directory-listing --directory-listing-format auto --open
```

Versioned assets (`app.3f9a2c.js`) get the one-year `max-age` from the extension table. HTML defaults to `no-cache`, so entry points revalidate via ETag and pick up new asset names.

## Checklist for Changes

- [ ] Header changes respect pipeline order and leave custom headers last (`design/SKILL.md`)
- [ ] Cache-Control arrays remain sorted and the 2xx/304 rule holds
- [ ] Pre-compressed and dynamic compression never double-encode
- [ ] `HEAD` returns the same headers as `GET` without a body
- [ ] Tests cover the feature-on and feature-off paths
