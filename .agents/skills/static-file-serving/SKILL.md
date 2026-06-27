---
name: static-file-serving
description: Serve static files and web assets with optimal headers, MIME types, compression, and caching for the Static Web Server (SWS) project
---

# Static File Serving Standards

Load this skill when configuring MIME types, selecting compression formats, setting cache headers, organizing static file directories, or optimizing file delivery with SWS.

**When to load**: documenting or reviewing compression setup, cache-control strategy, MIME-type overrides, SPA vs. multi-page file layout, directory listing, or pre-compression build steps.

## General Principles

- **Serve the right Content-Type**: SWS uses `mime_guess` to determine MIME types from file extensions. If the wrong type is served, rename the file or add a custom header rule
- **Compress where it helps**: Text-based formats compress well (HTML, CSS, JS, JSON, SVG, XML). Pre-compress at build time for zero-CPU serving
- **Cache aggressively for versioned assets**: Use fingerprinting in filenames (`app.a1b2c3d.js`) with `Cache-Control: max-age=31536000` (1 year)
- **Don't cache HTML entry points**: HTML files that reference versioned assets should have short or no cache TTL
- **Use a CDN for production**: Put SWS behind a CDN (Cloudflare, Fastly, CloudFront) for edge caching and DDoS protection

## MIME Types

SWS determines `Content-Type` from file extension. Ensure files have correct extensions:

| Extension | MIME Type | Category |
|-----------|-----------|----------|
| `.html`, `.htm` | `text/html` | Document |
| `.css` | `text/css` | Stylesheet |
| `.js`, `.mjs` | `text/javascript` (or `application/javascript`) | Script |
| `.json` | `application/json` | Data |
| `.xml` | `application/xml` | Data |
| `.svg` | `image/svg+xml` | Vector image |
| `.png` | `image/png` | Raster image |
| `.jpg`, `.jpeg` | `image/jpeg` | Raster image |
| `.webp` | `image/webp` | Raster image |
| `.avif` | `image/avif` | Raster image |
| `.gif` | `image/gif` | Raster image |
| `.ico` | `image/x-icon` | Icon |
| `.woff2` | `font/woff2` | Web font |
| `.woff` | `font/woff` | Web font |
| `.pdf` | `application/pdf` | Document |
| `.wasm` | `application/wasm` | WebAssembly |
| `.txt` | `text/plain` | Text |
| `.md` | `text/markdown` | Markdown |
| `.zip` | `application/zip` | Archive |
| `.tar` | `application/x-tar` | Archive |
| `.gz` | `application/gzip` | Compressed |
| `.br` | `application/brotli` | Compressed |
| `.zst` | `application/zstd` | Compressed |

### Custom MIME Types

Use the TOML config file to override or add MIME types for specific paths:

```toml
[advanced.headers]
source = "**/*.yaml"
headers = { Content-Type = "application/yaml" }
```

## Compression

### Static (Pre-compressed) Compression

Pre-compress files at build time. SWS serves `.br`, `.gz`, or `.zst` variants automatically based on the client's `Accept-Encoding` header:

| Variant | Extension | Encoding Header | Build Command |
|---------|-----------|-----------------|---------------|
| Brotli | `.br` | `br` | `brotli -q 11 -f file` |
| Gzip | `.gz` | `gzip` | `gzip -9 -k file` |
| Zstandard | `.zst` | `zstd` | `zstd -19 -k file` |

**Compression ratios** (typical for text files):

| Format | Level | Ratio | Speed |
|--------|-------|-------|-------|
| Brotli | 11 | ~75% | Slowest |
| Zstandard | 19 | ~72% | Medium |
| Gzip | 9 | ~68% | Fastest |

### Dynamic (On-the-fly) Compression

SWS compresses responses in real-time when the client sends `Accept-Encoding` and no pre-compressed variant exists:

- Only text-based MIME types are compressed (HTML, CSS, JS, JSON, XML, SVG, etc.)
- Responses below 860 bytes skip compression (overhead exceeds savings)
- Compression level is configurable: `fastest`, `default`, `best`

### SWS CLI Examples

```bash
# Serve with Brotli pre-compressed variants only (no on-the-fly compression)
static-web-server --root ./dist --compression-static --compression=false

# Serve with both pre-compressed variants and on-the-fly gzip fallback
static-web-server --root ./dist --compression-static --compression

# Serve with only on-the-fly compression at fastest level
static-web-server --root ./dist --compression --compression-level fastest
```

## Caching Strategy

### Cache-Control Headers

SWS sets `Cache-Control` based on file extension. Enable with `--cache-control-headers` (default: enabled):

| Category | Extensions | `max-age` |
|----------|-----------|-----------|
| Static assets | `.css`, `.js`, `.png`, `.woff2`, `.avif`, `.webp`, `.pdf`, `.ico`, `.gz`, `.bz2`, `.zip`, `.tar` | **1 year** (31536000) |
| Data/feeds | `.json`, `.xml`, `.rss`, `.atom` | **1 hour** (3600) |
| Everything else | unknown extensions | **1 hour** (3600) |

### Custom Cache Headers

Override `Cache-Control` for specific paths via TOML:

```toml
[advanced.headers]
source = "**/*.html"
headers = { Cache-Control = "no-cache" }
```

### Versioned Assets Pattern

For production deployments, use content-hash filenames and cache aggressively:

```
dist/
  index.html              ← short cache (or no-cache)
  assets/
    app.a1b2c3d.js        ← 1-year cache (fingerprinted)
    style.e4f5g6h.css     ← 1-year cache (fingerprinted)
    logo.h7i8j9k.png      ← 1-year cache (fingerprinted)
```

This way, when `app.a1b2c3d.js` changes, the new filename `app.x9y0z1.js` triggers a fresh download. The HTML entry point changes to reference the new filename.

## Security Headers for Static Sites

Enable `--security-headers` (auto-enabled with `--tls`) for production static sites:

```bash
static-web-server \
    --root ./dist \
    --tls --tls-cert cert.pem --tls-key key.pem \
    --security-headers \
    --cache-control-headers
```

Headers sent: `Strict-Transport-Security`, `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Content-Security-Policy: frame-ancestors 'self'`, `Referrer-Policy: strict-origin-when-cross-origin`.

## File Organization

### Single-Page Application (SPA)

```
dist/
  index.html              ← entry point (served for all routes)
  assets/
    app.js
    style.css
  favicon.ico
  robots.txt
```

SWS configuration:
```bash
static-web-server --root ./dist --page-fallback ./index.html
```

The `--page-fallback` option serves `index.html` for any 404, enabling client-side routing.

### Multi-Page Static Site

```
dist/
  index.html
  about.html
  blog/
    index.html
    post-1.html
  assets/
    main.css
    main.js
  images/
    hero.png
```

SWS configuration:
```bash
static-web-server --root ./dist --index-files "index.html,index.htm"
```

### Directory Listing (Development)

For development or internal tools, enable directory listing:

```bash
static-web-server --root ./public --directory-listing
```

Options: `--directory-listing-order` (0–6), `--directory-listing-format html|json`, `--directory-listing-download targz`.

## Checklist

- [ ] Do all files have correct extensions for MIME type detection?
- [ ] Are pre-compressed variants (`.br`/`.gz`/`.zst`) generated at build time?
- [ ] Are versioned assets using fingerprint filenames for cache busting?
- [ ] Is `Cache-Control` appropriate for the content type?
- [ ] Is the HTML entry point not cached (or has short TTL)?
- [ ] Are security headers enabled for production?
- [ ] Is TLS enabled for production deployments?
