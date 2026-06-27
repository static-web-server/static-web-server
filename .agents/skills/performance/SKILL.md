---
name: performance
description: Optimize or review performance for the Static Web Server (SWS) project — profiling, bottlenecks, resource usage, compression, and caching
---

# Performance Optimization

Load this skill when profiling, optimizing, or reviewing code for performance — latency, throughput, memory, or CPU.

**When to load**: a change touches the request hot path (`handler.rs`, `static_files/`, `compression.rs`, `fs/stream.rs`), a benchmark is being added or interpreted, a regression in requests/sec or latency is suspected, or a new dependency may affect binary size or runtime cost.

## General Approach

- **Measure before optimizing**: Use profiling tools (perf, flamegraph) to identify bottlenecks. Never optimize based on intuition
- **Set a target**: Define acceptable latency/throughput before starting. Stop optimizing when the target is met
- **Optimize the hot path**: Focus on code that runs on every request. Startup code and config parsing are low priority

## Rust Performance

- **Profile with `perf` and `flamegraph`**: `cargo flamegraph --bin static-web-server` for CPU profiles. Profile under load (e.g., `wrk` or `bombardier`)
- **Profile heap allocations with DHAT**: Use [DHAT](https://www.valgrind.org/docs/manual/dh-manual.html) or [dhat-rs](https://crates.io/crates/dhat) to find hot allocation sites. Reducing 10 allocations per million instructions can have measurable impact
- **Avoid unnecessary allocations**: Prefer `&Path` over `&PathBuf`, `&[u8]` over `Vec<u8>`, pass by reference where ownership is not needed
- **Pre-compute at startup**: Canonicalize paths, parse config, compile regex patterns, build Aho-Corasick automata once. Never on the request path
- **Pre-allocate collections**: Use `Vec::with_capacity`, `String::with_capacity` when the size is known
- **Stream large responses**: Use `tokio::fs::File` + `tokio::io::copy` for file serving. Never buffer the full file in memory
- **Inline small hot functions**: Use `#[inline]` on small functions called on every request (e.g., header name normalization, MIME lookups). Use `#[cold]` on error-path functions to guide branch prediction away from the hot path
- **Prefer `filter_map` over `filter().map()`**: Avoids an intermediate layer in hot iterator chains
- **Use `iter().copied()` for small types**: When iterating over `&u8`, `&u32`, etc., `.copied()` lets LLVM generate better code than receiving references
- **Use `chunks_exact` when chunk size evenly divides length**: Faster than `chunks` because it eliminates a remainder check per iteration
- **Prefer `ok_or_else` over `ok_or`**: `ok_or(expensive())` always evaluates its argument. `ok_or_else(|| expensive())` is lazy and only evaluates on `None`
- **Eliminate bounds checks in hot loops**: Use iteration instead of index-based access, or add an upfront assertion on the range to let the compiler prove bounds are safe

## HTTP Performance

### Connection Handling

- **HTTP/1.1 keep-alive**: Enabled by default via Hyper. Reduces connection setup overhead for subsequent requests
- **HTTP/2 multiplexing**: Enable with `--http2 --tls`. Multiple concurrent streams over a single TCP connection
- **Worker threads**: Default is `num_cpus * 1`. Increase `--threads-multiplier` for workloads with mixed CPU and I/O blocking (e.g., many concurrent clients with dynamic compression enabled — compression per-request is CPU-bound but high concurrency adds I/O wait interleaving). For pure CPU-bound workloads with minimal I/O, increasing threads beyond CPU count rarely helps.
- **Max blocking threads**: Default 512. For I/O-heavy patterns (large file serving), this is sufficient
- **Graceful shutdown**: Use `--grace-period` to allow in-flight requests to complete before shutdown

### Compression Tradeoffs

- **Static compression is free**: Pre-compressed `.br`/`.gz`/`.zst` files are served with zero CPU. Always prefer this for production
- **Dynamic compression overhead**: On-the-fly compression trades CPU for bandwidth. Use `--compression-level fastest` for high-traffic sites
- **Minimum size threshold**: Responses below 860 bytes skip dynamic compression entirely — the overhead exceeds any bandwidth savings
- **Compression algorithm priority** (by compression ratio × speed): zstd > brotli > gzip > deflate. zstd offers the best ratio-speed tradeoff

### Caching Headers

- **Cache-Control is enabled by default**: SWS sets `max-age` based on file extension:
  - 1 year for static assets (`.css`, `.js`, `.png`, `.woff2`, etc.)
  - 1 hour for feeds/API (`.json`, `.xml`, `.rss`, `.atom`)
  - 1 hour fallback for unknown extensions
  
You can override these defaults per file or extension using the configuration file. The above values are defaults, not hardcoded limits.
- **Conditional requests**: SWS supports `If-Modified-Since` and `If-Unmodified-Since` via `ConditionalHeaders`. Returns 304 when the file hasn't changed
- **ETag not implemented**: SWS uses `Last-Modified` instead. For byte-level cache validation, put SWS behind a CDN or reverse proxy

## File I/O Performance

### Buffering

- **Optimal buffer size**: `optimal_buf_size()` selects the best buffer size based on file metadata (uses `std::fs::Metadata::blksize()` when available)
- **`BufReader` with `take()`**: For byte-range requests, a `BufReader` wraps the file handle and limits bytes read to the requested range
- **Streaming avoids full-file buffering**: `FileStream` reads in chunks. The response body is a stream, not a byte buffer

### Path Operations

- **Canonicalize once at startup**: The root directory is canonicalized in `server/opts.rs`. Per-request path resolution reuses this
- **`try_metadata()` caches nothing**: Each call (in `src/fs/meta.rs`) is a filesystem syscall. The experimental memory cache feature (`mini-moka`, in `src/mem_cache/`) caches file metadata and content
- **Avoid `clone()` in the hot path**: `static_files.rs` avoids cloning file paths for non-directory requests

### Pre-compressed Static Files

- **Zero-CPU serving**: SWS detects `.br`/`.gz`/`.zst` variants via `Accept-Encoding` and serves them directly. No compression step runs
- **Build-time pre-compression**: Generate variants with maximum quality: `brotli -q 11`, `gzip -9`, `zstd -19`. SWS serves them as-is
- **Vary header**: `Vary: Accept-Encoding` is appended so caches know to store multiple variants

## Memory

- **Minimal per-connection state**: SWS stores only the remote address and handler opts (shared via `Arc`). No per-connection buffers
- **Response body is a stream**: File contents are streamed, not buffered. Exception: small generated responses (health endpoint, error pages, directory listing HTML)
- **Experimental in-memory cache**: `mini-moka` (in `src/mem_cache/`) caches hot files in memory with LFU admission and LRU eviction. Configurable `capacity` (default 100 entries), `ttl` (default 1800s), `tti`, and `max_file_size`. Keys use `CompactString` to reduce allocation

## Allocation Patterns

- **Prefer `clone_from` over reassign-and-clone**: `a.clone_from(&b)` reuses `a`'s existing heap allocation when possible, avoiding an extra alloc/free. Especially valuable for `Vec` and `String` in hot loops
- **Reuse collections across iterations**: Declare the collection outside the loop, call `.clear()` at the end of each iteration. Avoids repeated alloc/free while keeping the heap allocation alive
- **Use `Cow<'_, str>` / `Cow<'_, Path>` for mixed borrowed/owned data**: Avoids allocating a `String`/`PathBuf` when the data is already a static literal or an existing slice that won't be modified
- **Use `SmallVec<[T; N]>` for short, stack-like sequences**: When most allocations hold ≤ N elements (e.g., header value lists, index file candidates), `smallvec` avoids heap allocation entirely for the common case
- **Convert finalized `Vec` to `Box<[T]>` with `into_boxed_slice()`**: Drops the unused capacity word, shrinking the type from 3 words to 2. Good for config-time data that is built once and never grown
- **Return `impl Iterator<Item=T>` instead of `Vec<T>` from helpers**: Avoids an allocation when the caller only needs to iterate
- **Avoid `format!` when a literal or `write!` suffices**: Every `format!` call allocates a `String`. Write directly to a `&mut String` or use `std::fmt::Write` instead

## Type Sizes

- **Keep hot types under 128 bytes**: The compiler emits `memcpy` for values larger than 128 bytes. If a hot type exceeds this, check its layout with `RUSTFLAGS=-Zprint-type-sizes cargo +nightly build --release`
- **Box large enum variants**: If one variant is much larger than the others, box its fields to bring all variants to a similar small size. Reduces stack pressure and cache churn
- **Use smaller integer types for index/count fields**: Prefer `u32` over `usize` for counts and offsets stored in frequently instantiated structs (e.g., header tables, path segments). Cast to `usize` at use sites
- **Assert type sizes in tests**: Add `static_assertions::assert_eq_size!(HotType, [u8; N]);` for performance-critical types so that accidental size regressions cause a compile error

## Release Build Configuration

The default `cargo build --release` profile is a good starting point, but the following options can improve throughput for production SWS builds:

| Option | Effect | Cargo.toml |
|--------|--------|-----------|
| `lto = "thin"` | Cross-crate inlining, 5–15% speedup, moderate compile cost | `[profile.release]` |
| `codegen-units = 1` | Single codegen unit, enables more optimizations, slower compile | `[profile.release]` |
| `panic = "abort"` | Removes unwinding machinery, smaller binary, slight speedup | `[profile.release]` |

For a custom server build where broad CPU compatibility is not required:
```
RUSTFLAGS="-C target-cpu=native" cargo build --release
```
This emits AVX/SSE instructions optimal for the build machine, which can improve compression throughput.

> Note: `target-cpu=native` produces a non-portable binary. Do not use for distributed release artifacts.

## Benchmarking

### Tools

- **HTTP load generators**: `wrk`, `bombardier`, `oha`, `hey`
- **CPU profiling**: `perf record` + `flamegraph`, `cargo flamegraph`, `cargo instruments` (macOS), `samply` (cross-platform)
- **Allocation profiling**: `dhat-rs` (all platforms), DHAT via Valgrind (Linux) — identifies hot allocation sites
- **Monitoring**: Prometheus metrics via `--metrics` + `/metrics` endpoint

### What to Measure

- **Requests per second** at concurrency levels: 1, 10, 100, 1000
- **Latency percentiles**: p50, p95, p99
- **Memory usage**: RSS before and under load
- **CPU utilization**: Per-core usage during load test
- **Allocation rate**: Use DHAT to confirm per-request allocations are not growing unexpectedly

## Checklist

- [ ] Is there a benchmark or profile showing the bottleneck?
- [ ] Are paths canonicalized once, not per-request?
- [ ] Is static compression used where possible (zero CPU)?
- [ ] Is dynamic compression size-threshold applied (860 bytes)?
- [ ] Are large files streamed, not buffered?
- [ ] Are Cache-Control headers set appropriately for the content type?
- [ ] Are worker threads configured for the workload?
- [ ] Is keep-alive or HTTP/2 enabled for connection reuse?
- [ ] Are hot allocation sites identified with DHAT or similar?
- [ ] Are collections pre-allocated or reused rather than recreated per request?
- [ ] Are `clone()` calls on the hot path justified — or replaceable with `clone_from`, `Cow`, or a reference?
- [ ] Are hot types under 128 bytes (no unintended `memcpy`)?
- [ ] Are `#[inline]` / `#[cold]` attributes applied where profiling shows they help?
- [ ] Are `ok_or_else` / lazy combinators used instead of eager alternatives in hot paths?
