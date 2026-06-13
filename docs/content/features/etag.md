# ETag

**`SWS`** provides weak [`ETag`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag) support, enabling browsers and CDNs to revalidate cached static assets with a single lightweight round-trip instead of re-downloading unchanged files.

The validator is derived from the file's `mtime` (last modification time, in nanoseconds) and `size` (in bytes) — no file hashing or I/O beyond the `stat()` call. The ETag is always emitted with the weak prefix (`W/`) using the following format:

```text
W/"<mtime_hex>-<len_hex>"
```

!!! info "Weak vs strong"

    A weak ETag indicates that two representations are *semantically equivalent* even if byte-for-byte different. This is correct for content negotiation scenarios — for example, a pre-compressed `.br` variant of a file shares the same weak ETag as the original. SWS emits `Vary: Accept-Encoding` so that intermediary caches key variants separately.

This feature is enabled by default and can be controlled by the boolean `--etag` option or the equivalent [SERVER_ETAG](../configuration/environment-variables.md#server_etag) env.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --etag true
```

## How it works

When a client requests a static file for the first time, SWS reads the file's metadata from the filesystem, builds a weak ETag from its `mtime` and `size`, and includes it in the `200 OK` response:

```http
HTTP/1.1 200 OK
Content-Type: text/html
Content-Length: 12345
Last-Modified: Thu, 29 May 2026 12:00:00 GMT
ETag: W/"18d4a2b1c3deadbeef-3039"
Cache-Control: no-cache
Accept-Ranges: bytes
```

On subsequent requests, the client sends conditional headers that SWS evaluates against the file's current ETag. If the file hasn't changed, SWS responds with `304 Not Modified` — **zero body bytes** are transferred:

```http
GET /index.html HTTP/1.1
Host: example.com
If-None-Match: W/"18d4a2b1c3deadbeef-3039"
```

```http
HTTP/1.1 304 Not Modified
ETag: W/"18d4a2b1c3deadbeef-3039"
Last-Modified: Thu, 29 May 2026 12:00:00 GMT
Cache-Control: no-cache
```

When the file changes (different `mtime` or `size`), the ETag no longer matches and SWS sends a fresh `200 OK` with the new validator.

## Conditional headers supported

SWS implements the full [RFC 7232 §6](https://datatracker.ietf.org/doc/html/rfc7232#section-6) precedence rules for evaluating conditional request headers against the resource's ETag and `Last-Modified` validators.

| Client header | Condition | SWS response |
| -- | -- | -- |
| *(none)* | First request, no validators held | `200 OK` with `ETag` and `Last-Modified` headers |
| `If-None-Match: W/"..."` | ETag matches (resource unchanged) | `304 Not Modified` — validators echoed, empty body |
| `If-None-Match: W/"..."` | ETag differs (resource changed) | `200 OK` with fresh body and new ETag |
| `If-None-Match: *` | Any representation exists | `304 Not Modified` |
| `If-None-Match: *, W/"..."` | Wildcard present in list | `304 Not Modified` (wildcard matched first per RFC 7232) |
| `If-Match: *` | Any representation exists | `200 OK` |
| `If-Match: W/"..."` | Weak ETag sent | `412 Precondition Failed` — weak validators never satisfy strong comparison (RFC 7232 §3.1) |
| `If-Match: "..."` | Strong ETag sent by client | SWS matches if the strong ETag equals SWS's weak ETag without the `W/` prefix |
| `If-Range: W/"..."` | Weak ETag in range request | Falls back to `200 OK` — weak validators cannot strongly match for range (RFC 7233 §3.2); `Range` header is ignored, full file served |
| `If-Modified-Since` | No `If-None-Match` present | Date-based comparison (fallback when ETag is absent or disabled) |
| `If-Unmodified-Since` | No `If-Match` present | Date-based comparison (fallback when ETag is absent or disabled) |

### Precedence order

When multiple conditional headers are present in the same request, SWS evaluates them in the order defined by RFC 7232 §6:

1. **`If-Match`** — if present and fails then it returns `412`. Otherwise, continue.
2. **`If-Unmodified-Since`** — only evaluated when `If-Match` is absent. If fails then it returns `412`.
3. **`If-None-Match`** — if present and matches then it returns `304`. Otherwise, continue.
4. **`If-Modified-Since`** — only evaluated when `If-None-Match` is absent. If matches then it returns `304`.
5. **`If-Range`** — evaluated last, informs whether to serve a partial `206` or full `200`.

## ETag with `--cache-control-headers`

The `--etag` and `--cache-control-headers` features are orthogonal and coexist on the same response:

- SWS's static files handler emits the `ETag` header.
- The post-processing [Cache-Control Headers](cache-control-headers.md) pipeline appends `Cache-Control`.

When both are enabled (default), responses include both headers:

```http
HTTP/1.1 200 OK
Content-Type: text/html
ETag: W/"18d4a2b1c3deadbeef-3039"
Cache-Control: no-cache
```

For HTML files, `Cache-Control: no-cache` causes browsers to revalidate on every request. The ETag turns nearly all of those revalidations into lightweight `304` responses with zero body bytes.

## ETag with static compression

SWS's [pre-compressed files serving](compression-static.md) emits the same weak ETag for a file and its compressed variants (`.br`, `.gz`, `.zst`). This is correct because:

- Weak ETag semantics signal that the representations are semantically equivalent.
- `Vary: Accept-Encoding` ensures intermediary caches key variants separately.

## ETag with the in-memory cache

When the [In-Memory Cache](memory-cache.md) is enabled, the ETag `HeaderValue` is built once when the file is first read from disk and stored alongside the body bytes. Every subsequent cache hit clones the pre-built value. The typed ETag needed for conditional comparison is parsed lazily, only when the request actually carries `If-None-Match`, `If-Match`, or `If-Range`.

## Disabling the feature

To omit the `ETag` header from all responses:

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --etag false
```

When disabled, SWS falls back to date-based validation via `Last-Modified`, `If-Modified-Since`, and `If-Unmodified-Since`.
