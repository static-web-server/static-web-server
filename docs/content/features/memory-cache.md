# In-Memory Cache

**`SWS`** provides an optional file-level in-memory cache that stores hot files in RAM, serving them directly without touching the filesystem on repeat requests.

The cache uses a **Least Frequently Used (LFU)** admission policy and a **Least Recently Used (LRU)** eviction policy. Every entry has a **Time-to-Live (TTL)** and a **Time-to-Idle (TTI)** — expired or idle entries are evicted automatically.

This feature is **enabled by default** via the `mem-cache` Cargo feature and configured entirely through the TOML configuration file.

## Cargo Feature

The in-memory cache is gated by the `mem-cache` Cargo feature, which is included in the `default` feature set.

To **disable** it, build without default features and select only what you need:

```sh
cargo build --release --no-default-features \
  --features "compression,http2,tls-ring,directory-listing,fallback-page,basic-auth,metrics"
```

Omitting `mem-cache` removes the `compact_str` and `mini-moka` dependencies from the binary.

## Configuration

Add the `[advanced.memory-cache]` section to your TOML configuration file. All fields are optional and will fall back to their defaults if omitted.

```toml
[advanced.memory-cache]
capacity = 100
# 30min
ttl = 1800
# 5min
tti = 300
# 8mb
max-file-size = 8192
```

### Fields

| Field | Type | Default | Maximum | Unit | Description |
| -- | -- | -- | -- | -- | -- |
| `capacity` | `u64` | 100 | 100 000 | entries | Maximum number of cached file entries. |
| `ttl` | `u64` | 1800 | 86 400 | seconds | Time-to-Live: maximum age of a cached entry before eviction. |
| `tti` | `u64` | 300 | 3 600 | seconds | Time-to-Idle: maximum idle time before an entry is evicted. |
| `max-file-size` | `u64` | 8192 | 32 768 | KiB | Maximum file size allowed into the cache. Files exceeding this limit are served from disk. |

!!! warning "Defaults & Limits"

    Values exceeding the maximum limits are clamped silently. For example, setting `capacity = 200000` results in `100000`.

## How it works

1. **First request**: SWS reads the file from disk, streams the response to the client, and simultaneously populates the cache store.
2. **Subsequent requests** (within TTL/TTI): SWS serves the response directly from memory — zero disk I/O.
3. **Concurrent misses**: The underlying cache store is fully concurrent and thread-safe. Brief duplicate reads under contention are benign and resolve themselves once the entry is populated.
4. **Eviction**: Entries that exceed TTL, idle beyond TTI, or are displaced by the LFU/LRU policies are evicted automatically.

!!! tip "Pair with static compression"

    The in-memory cache stores **uncompressed** file data. It is safe to combine with [on-the-fly compression](compression.md) and [pre-compressed file serving](compression-static.md) — the cache lookup happens before compression in the request pipeline.

!!! note "Partial content (range requests)"

    Byte-range requests (`Range` header) are supported. The cache stores the whole file; partial responses are sliced from the cached bytes on every request.

## X-Cache response header

When a response is served from the in-memory cache, SWS includes the standard **`X-Cache: HIT`** header. This is the same header used by nginx, Apache, Varnish, and most CDNs to indicate cache status.

- **Cache hit** — the response includes `X-Cache: HIT`.
- **Cache miss** (first request, or cache disabled) — the `X-Cache` header is **not present**.

You can use this header to verify cache behaviour via `curl`:

```sh
# First request (miss — no X-Cache header):
curl -sI http://localhost:8787/index.html | grep -i x-cache
# (no output)

# Second request (hit — X-Cache: HIT):
curl -sI http://localhost:8787/index.html | grep -i x-cache
# x-cache: HIT
```

Or inspect it in browser developer tools under the **Network → Response Headers** tab.

## Performance considerations

- For **small, frequently accessed files** (favicons, CSS, JS, HTML), enable the cache with a moderate `max-file-size` (e.g., 8192 KiB) and a long TTL (e.g., 3600s).
- For **large files** (video, archives), keep `max-file-size` low or disable the cache entirely — serving large files from memory can increase RSS significantly.
- The cache store is lock-free for reads and uses fine-grained internal sharding for writes, so it scales well under high concurrency without blocking other requests.
