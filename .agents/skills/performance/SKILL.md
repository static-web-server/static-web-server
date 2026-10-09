---
name: performance
description: Measure, optimize, or review performance in the Static Web Server (SWS) project — request hot path, syscalls, allocations, compression cost, caching, thread configuration, binary size, CodSpeed benchmarks, and profiling. Use when a change touches handler.rs, static_files/, fs/, compression, header steps, or adds a dependency, or when investigating a latency, throughput, or memory regression.
---

# Performance

SWS optimizes for low per-request syscalls and allocations on the static-file path. Changes are justified by a benchmark or profile, not intuition.

**When to load**: editing the hot path (`handler.rs`, `static_files/`, `fs/`, `compression*.rs`, `control_headers.rs`, `security_headers.rs`, `custom_headers.rs`, `exts/`), adding a bench, interpreting a CodSpeed result, or adding a dependency.

## Workflow

1. **Reproduce and measure** with a bench in `benches/` or a load test against a release build
2. **Profile** to find where time goes (see Tools)
3. **Change one thing**, re-measure, and keep the change only if the gain is measurable
4. **Add or update a bench** so CodSpeed catches regressions on future PRs
5. **Never trade correctness or security for speed**. Caching a security decision needs a bounded staleness window (see the containment cache)

## What the Hot Path Already Does

Know these before optimizing; don't undo them:

- **Startup pre-computation**: root canonicalized once (`server/opts.rs`); glob/regex rules and Aho-Corasick placeholder replacers compiled in `settings/`; static `HeaderValue`s built with `from_static`
- **Containment cache**: `enforce_containment` was ~18% of inclusive CPU on the static path because of `canonicalize()`. A per-thread `thread_local!` cache of proven-safe paths with wholesale TTL eviction removes the syscall in steady state (`static_files/security.rs`)
- **Single open + fstat**: `fs::meta::try_file_open` opens the file and reads metadata from the descriptor instead of `stat` then `open`
- **Cheap checks first**: method check and pre-processing steps return early; symlink walking (one `symlink_metadata` per component) runs only when `--follow-symlinks` is off
- **Zero-alloc header lookups**: `control_headers::get_cache_control_header` lowercases the extension into a 16-byte stack buffer and uses `binary_search` over sorted arrays
- **Streaming**: file bodies are streamed with a buffer from `fs::stream::optimal_buf_size()` (device block size, capped at file length). Only small generated bodies are buffered
- **Pre-compressed first**: `.br`/`.gz`/`.zst` variants cost no CPU; dynamic compression skips responses that already have `Content-Encoding`, are not compressible, or are under 200 bytes
- **Memory cache** (`mem-cache`, opt-in via `[advanced.memory-cache]`): `mini-moka` with `CompactString` keys; caches small files (default max 8 MiB, capped 32 MiB) for `ttl`/`tti`
- **Allocator**: `mimalloc` on 64-bit musl targets (`src/bin/server.rs`)

## Hot-Path Rules

- No per-request `canonicalize`, regex compilation, glob building, or config parsing
- No `format!` or `String` building for header values that can be `from_static` or written into a reused buffer
- Prefer borrowing (`&str`, `&Path`, `Cow`) over cloning; justify each `clone()` on the request path
- Prefer `ok_or_else`/`unwrap_or_else` over eager variants when the fallback allocates
- Use `#[inline]` for small functions called per request when a bench shows it helps; `#[cold]` on error-path helpers
- Keep per-request work proportional to the request, not to the config (e.g., don't iterate all header rules when an indexed lookup works)
- Don't add a syscall to the fast path to make a cache "more precise" without measuring the cost

## Threads and Runtime

- Worker threads = available CPUs × `--threads-multiplier` (default `1`; `0` and `1` both mean "CPU count")
- `--max-blocking-threads` (default `512`) bounds tokio's blocking pool used by file I/O
- Raising the multiplier helps when dynamic compression (CPU) and many concurrent clients interleave; for pure static serving the CPU count is usually optimal. Measure before recommending a change

## Release Build

`Cargo.toml` already sets `[profile.release]`: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `opt-level = 3`, `strip = true`. Don't propose these as improvements.

For profiling, use the `release-profiling` profile (same optimizations, with debug info, not stripped):

```bash
cargo build --profile release-profiling --features all
samply record target/release-profiling/static-web-server -p 8080 -d docker/public/
```

`RUSTFLAGS="-C target-cpu=native"` produces a non-portable binary; local experiments only.

## Benchmarks

### Micro-benchmarks (CodSpeed)

`benches/` is a standalone crate using `codspeed-criterion-compat`. Current suites: `control_headers`, `static_files`, `response_headers`, `http_ext`, `security_sanitize_path`, `security_basic_auth`, `security_redirects`. Every PR runs them in CodSpeed CPU simulation (`.github/workflows/codspeed.yml`) and reports deltas.

```bash
cd benches
cargo bench                                     # local Criterion timings
cargo bench --bench control_headers             # one suite
cargo codspeed build && codspeed run --mode simulation -- cargo codspeed run
```

Benches call public functions (e.g. `control_headers::append_headers`, `static_files::handle`). If a hot function is private, expose a thin `pub` or `#[doc(hidden)] pub` entry point rather than benchmarking through the whole server. Register new suites as `[[bench]]` entries with `harness = false` in `benches/Cargo.toml`.

### Load tests

- Maintainers trigger `.github/workflows/perfcheck.yml` by commenting `/try+ perfcheck` on a PR. It runs `vegeta` (4 workers, 100 connections, 10s) at several rates and reports latency percentiles
- Locally: `oha`, `wrk`, `bombardier`, or `vegeta` against a release build. Report requests/sec and p50/p95/p99 at fixed concurrency, plus RSS before and under load

### Tools

- CPU: `samply`, `perf record` + flamegraph, `cargo flamegraph`
- Allocations: `dhat-rs` or Valgrind DHAT
- Syscalls: `strace -c -f` on Linux to count per-request syscalls
- Type layout: `RUSTFLAGS=-Zprint-type-sizes cargo +nightly build --release` for types passed by value on the hot path
- Runtime metrics: `--metrics` exposes Prometheus counters; the `experimental` feature adds tokio runtime metrics

## Binary Size

Release binary is about 4MB. Before adding a dependency, build `--release --features all` before and after and compare. Growth above 100KB needs a feature flag or must replace existing code. Use `default-features = false` and enable only what is needed.

## Review Checklist

- [ ] A bench or profile shows the problem and the improvement
- [ ] A bench covers the changed hot-path function
- [ ] No new per-request syscalls, canonicalization, or regex/glob compilation
- [ ] No new allocations in the steady-state path without justification
- [ ] Caches have bounded size and bounded staleness, and don't weaken security checks
- [ ] Binary size impact checked for new dependencies
