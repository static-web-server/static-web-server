---
name: static-file-serving
description: Explain or change how the Static Web Server (SWS) serves files — index resolution, MIME types, pre-compressed and on-the-fly compression, Cache-Control, ETag and conditional requests, byte ranges, directory listing, SPA fallback, markdown negotiation, memory cache, and TOML headers/rewrites/redirects/virtual hosts. Use when working on static_files/, compression, control_headers, custom_headers, directory_listing, mem_cache, or writing user-facing config examples.
---

# Static File Serving

How SWS turns a request path into a response, and how users configure it. Values are taken from the source; re-check the cited file before changing a default.

**When to load**: changing file resolution, compression, caching, conditional or range handling, directory listing, memory cache, or custom headers; answering "how do I serve X with SWS"; writing CLI or TOML examples.

## Resolution (`static_files::handle()`)

1. Method check: `GET`, `HEAD`, `OPTIONS`
2. `sanitize_path()` maps the URI path under the root (`security/SKILL.md`)
3. Memory cache lookup (`GET`/`HEAD`, only with `[advanced.memory-cache]`): a hit picks the cached variant matching `Accept-Encoding` and returns without disk access
4. `resolve::file_metadata()`: directory → each `--index-files` entry in order; missing file → `<path>.html`; with `--compression-static`, a pre-compressed sibling
5. `security::enforce()` on the resolved file and, separately, on the pre-compressed variant
6. Short-circuits: directory without trailing slash → 308 (`--redirect-trailing-slash`, default `true`); `OPTIONS` → 204 with `Allow` and `Accept-Ranges`; directory listing or archive download
7. File reply: conditional checks, byte range, streamed body

## MIME Types and Charset

- `Content-Type` from `mime_guess` on the extension. No override flag; use a custom header rule
- `--text-charset` (default `true`) appends `charset=utf-8` to `text/*` responses lacking one
- `exts/mime.rs` decides which types are compressible (`text/*`, `+json`/`+xml` suffixes, a fixed list of `application/*`)

## Compression

### Pre-compressed (`--compression-static`, default `true`)

For `file.ext`, SWS looks for `file.ext.br`, `.gz`, or `.zst` in the client's `Accept-Encoding` q-value order (`q=0` means not acceptable; `gzip` and `deflate` both map to `.gz`). The variant is served with `Content-Encoding` on success (not on 304/412/416) and `Vary: Accept-Encoding`; dynamic compression then skips it.

```bash
brotli -q 11 -k dist/app.js && gzip -9 -k dist/app.js && zstd -19 -k dist/app.js
```

### On-the-fly (`--compression`, default `true`)

`compression::auto()` encodes when all hold:
- method is not `OPTIONS`
- status is not 1xx, 204, 304, or 206, and there is no `Content-Range`
- no `Content-Encoding` yet
- the client accepts a compiled-in encoding (`zstd`, `br`, `gzip`, `deflate`)
- the MIME type is compressible
- `Content-Length` is absent or ≥ **200 bytes** (`MIN_COMPRESS_SIZE`)

`HEAD` gets the same `Content-Encoding` header as `GET` and no body. `--compression-level`: `fastest`, `default`, `best`; `default` is level 4 (gzip, deflate, brotli) or 3 (zstd).

## Cache-Control (`--cache-control-headers`, default `true`)

Set by URI extension (case-insensitive), **only for `2xx` and `304`**; every other status gets `no-cache`.

| Extensions | Value |
|-----------|-------|
| `avif bmp bz2 css doc gif gz htc ico jpeg jpg js jxl map mjs mp3 mp4 ogg ogv pdf png rar rtf tar tgz wav weba webm webp woff woff2 zip` | `max-age=31536000` |
| `atom rss` | `max-age=3600` |
| anything else (`html`, `json`, `svg`, no extension, …) | `no-cache` |

Both arrays in `control_headers.rs` must stay sorted (`binary_search`). Override per path with a custom header rule.

## ETag and Conditional Requests (`--etag`, default `true`)

- Weak validator `W/"<mtime_ns_hex>-<len_hex>"` (`etag.rs`)
- `conditional_headers.rs`: `If-Match` / `If-Unmodified-Since` → 412, then `If-None-Match` / `If-Modified-Since` → 304 (`If-None-Match` wins). `If-Range` honored for byte ranges
- `Last-Modified` sent when the filesystem reports a modification time

## Byte Ranges (`response/range.rs`)

`Range: bytes=` → 206 with `Content-Range`; unsatisfiable → 416. Partial responses are never dynamically compressed.

## Memory Cache (`[advanced.memory-cache]`, `mem-cache` feature)

Entries are keyed by `(path, ContentCoding)`, so a pre-compressed variant is cached under its own encoding and served only to clients that accept it. Only `GET`/`HEAD` hits are served; a cached entry whose evaluation fails (bad `Range`) falls through to the normal path. Hits carry `X-Cache: HIT`.

## Directory Listing (`--directory-listing`, default `false`)

- `--directory-listing-format html|json|auto`; `auto` picks via `Accept` and adds `Vary: Accept`
- `--directory-listing-order 0..5` (name/modified/size, asc/desc); `6` (default) unsorted; `?sort=N` per request
- `--directory-listing-download targz` adds a tar.gz link (`directory-listing-download` feature); archives exclude symlinks pointing outside the root

## Fallback and Error Pages

- `--page-fallback <file>` (`fallback-page` feature; path **not** relative to the root): served with 200 for `GET` requests that would 404, except missing assets, which keep 404: `Sec-Fetch-Dest` other than a document or frame, or, without that header, a non-HTML extension (`/favicon.ico`) not asking for `text/html`
- `--page404` (`./404.html`), `--page50x` (`./50x.html`): loaded at startup; relative paths resolve under the root; missing file → generic message

## Markdown Negotiation (`--accept-markdown`, default `false`)

- `Accept: text/markdown` → look for `<path>.md`, `<path>.html.md`, `<path>/index.html.md`; serve as `text/markdown`; hidden-file policy applies to the variant
- When enabled, HTML and markdown responses, directory listings (any format), a `304` whose resolved file is HTML, and a trailing-slash `308` carry `Vary: Accept`. Other assets and pre-processing short-circuits don't
- A `[[advanced.headers]]` rule that sets `Vary` replaces it

## Advanced TOML Rules

TOML-only, glob `source` matched against the URI path, arrays of tables:

```toml
[advanced]

# Custom headers: applied last, override earlier headers
[[advanced.headers]]
source = "/assets/**"
status = [200, 206, 304]       # optional; default: every status
headers = { Cache-Control = "public, max-age=31536000, immutable" }

[[advanced.headers]]
source = "**"
scope = "all"                  # default "files": static responses only.
                               # "all": also redirects, health, metrics, 401, 403, 405, maintenance
headers = { X-Request-Id = "static" }

# Redirects: short-circuit with 301 or 302; `host` is optional
[[advanced.redirects]]
host = "old.example.com"
source = "/pages/{*}.html"
destination = "https://example.com/?p=$1"
kind = 301

# Rewrites: change the path internally; add `redirect = 301` to redirect instead
[[advanced.rewrites]]
source = "/error-page/{404,50x}.html"
destination = "/$1.html"

# Virtual hosts: pick a root by Host
[[advanced.virtual-hosts]]
host = "example.com"
root = "./sites/example"

# In-memory cache; values above the maximums are clamped silently
[advanced.memory-cache]
capacity = 100        # entries (max 100000)
ttl = 1800            # seconds (max 86400)
tti = 300             # idle seconds (max 3600)
max-file-size = 8192  # KiB (default 8192, max 32768)
```

Globs compile through `globset` with `literal_separator(true)`, then into capturing regex groups (`settings/mod.rs`): `{*}` within one segment, `{**}` across segments, `{a,b}` alternation. `$1`, `$2`, … reference captures in pattern order. Working rules: `tests/fixtures/toml/rewrites.toml`, `redirects.toml`, `custom_headers_*.toml`.

## Common Setups

```bash
static-web-server -d ./dist --page-fallback ./dist/index.html              # SPA
static-web-server -d ./dist --tls --tls-cert cert.pem --tls-key key.pem    # TLS + HTTP/2, security headers, HSTS
static-web-server -d . --directory-listing --directory-listing-format auto --open
```

Versioned assets (`app.3f9a2c.js`) get the one-year `max-age`; HTML gets `no-cache` and revalidates via ETag.

## Checklist for Changes

- [ ] Header changes respect pipeline order and leave custom headers last (`design/SKILL.md`)
- [ ] Cache-Control arrays stay sorted; the 2xx/304 rule holds
- [ ] Pre-compressed, dynamic, and cached responses never double-encode or mislabel `Content-Encoding`
- [ ] `HEAD` returns `GET`'s headers without a body; `OPTIONS` and bodiless statuses get no `Content-Encoding`
- [ ] Tests cover feature-on and feature-off paths
