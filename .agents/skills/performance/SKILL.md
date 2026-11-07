---
name: performance
description: Measure, optimize, or review performance in the Static Web Server (SWS) project — request hot path, syscalls, allocations, compression cost, caching, thread configuration, binary size, CodSpeed benchmarks, and profiling. Use when a change touches handler.rs, static_files/, fs/, compression, header steps, access logs, or adds a dependency, or when investigating a latency, throughput, or memory regression.
---

# Performance

SWS optimizes for low per-request syscalls and allocations on the static-file path. Changes are justified by a benchmark or profile, not intuition.

**When to load**: editing the hot path (`handler.rs`, `static_files/`, `fs/`, `compression*.rs`, `control_headers.rs`, `security_headers.rs`, `custom_headers.rs`, `access_logs.rs`, `exts/`), adding a bench, reading a CodSpeed result, or adding a dependency.

## Workflow

1. Reproduce and measure with a bench in `benches/` or a load test against a release build
2. Profile to find where time goes
3. Change one thing, re-measure, keep it only if the gain is measurable
4. Add or update a bench so CodSpeed catches regressions
5. Never trade correctness or security for speed. A cached security decision needs a bounded staleness window (containment cache)

## What the Hot Path Already Does

Don't undo these:

- **Startup pre-computation**: root canonicalized once; glob/regex rules, Aho-Corasick replacers, and the access log format compiled in `settings/`; static `HeaderValue`s via `from_static`
- **Containment cache**: `canonicalize()` was ~18% of CPU on the static path. A `thread_local!` cache of proven-safe paths with wholesale TTL eviction removes the syscall in steady state (`static_files/security.rs`)
- **Single open + fstat**: `fs::meta::try_file_open` reads metadata from the descriptor
- **Cheap checks first**: method check and pre-processing return early; symlink walking runs only with `--follow-symlinks` off
- **Zero-alloc header lookups**: `control_headers` lowercases the extension into a 16-byte stack buffer and `binary_search`es sorted arrays
- **Streaming**: bodies streamed with `fs::stream::optimal_buf_size()` (device block size, capped at file length); only small generated bodies are buffered
- **Pre-compressed first**: `.br`/`.gz`/`.zst` cost no CPU; dynamic compression skips encoded, incompressible, partial, bodiless, and <200-byte responses
- **Memory cache** (`[advanced.memory-cache]`): `mini-moka`, `CompactString` keys with the content coding, so a hit serves the right variant without disk access; files up to 8 MiB by default (max 32 MiB), `ttl`/`tti`
- **Access log**: one line per request, written when the body is dropped; values escaped into a bounded buffer
- **Allocator**: `mimalloc` on 64-bit musl targets (`src/bin/server.rs`)

## Hot-Path Rules

- No per-request `canonicalize`, regex compilation, glob building, or config parsing
- No `format!`/`String` for header values that can be `from_static` or written into a reused buffer
- Borrow (`&str`, `&Path`, `Cow`) over clone; justify each `clone()` on the request path
- `ok_or_else`/`unwrap_or_else` when the fallback allocates
- `#[inline]` on small per-request functions when a bench shows it helps; `#[cold]` on error paths
- Per-request work proportional to the request, not the config (indexed lookups over iterating all rules)
- Don't add a syscall to make a cache more precise without measuring

## Threads and Runtime

- Workers = CPUs × `--threads-multiplier` (default `1`; `0` and `1` both mean CPU count)
- `--max-blocking-threads` (default `512`) bounds tokio's blocking pool used by file I/O
- Raising the multiplier helps when dynamic compression and many clients interleave; for pure static serving the CPU count is usually optimal. Measure first

## Release Build

`[profile.release]`: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `opt-level = 3`, `strip = true`. Don't propose these as improvements.

Profiling uses the `release-profiling` profile (debug info, not stripped):

```bash
cargo build --profile release-profiling --features all
samply record target/release-profiling/static-web-server -p 8080 -d docker/public/
```

`RUSTFLAGS="-C target-cpu=native"` is non-portable; local experiments only.

## Benchmarks

### Micro-benchmarks (CodSpeed)

`benches/` is a standalone crate (`codspeed-criterion-compat`). Suites: `control_headers`, `static_files`, `response_headers`, `http_ext`, `security_sanitize_path`, `security_basic_auth`, `security_redirects`. `.github/workflows/codspeed.yml` runs them per PR in CPU simulation and reports deltas.

```bash
cd benches
cargo bench                              # local Criterion timings
cargo bench --bench control_headers
```

Benches call public functions (`control_headers::append_headers`, `static_files::handle`). Expose a thin `pub` or `#[doc(hidden)] pub` entry point for a private hot function rather than benchmarking through the server. Register new suites as `[[bench]]` with `harness = false` in `benches/Cargo.toml`.

### Load tests

- Maintainers trigger `.github/workflows/perfcheck.yml` by commenting `/try+ perfcheck` on a PR (`vegeta`, 4 workers, 100 connections, 10 s, several rates)
- Locally: `oha`, `wrk`, `bombardier`, or `vegeta` against a release build. Report req/s and p50/p95/p99 at fixed concurrency, plus RSS before and under load

### Tools

CPU: `samply`, `perf` + flamegraph. Allocations: `dhat-rs`, Valgrind DHAT. Syscalls: `strace -c -f`. Type layout: `RUSTFLAGS=-Zprint-type-sizes cargo +nightly build --release`. Runtime: `--metrics` (Prometheus), `experimental` feature for tokio runtime metrics.

## Binary Size

About 4MB. Before adding a dependency, build `--release --features all` before and after and compare. Growth above 100KB needs a feature flag or must replace existing code. `default-features = false`, enable only what's needed.

## Review Checklist

- [ ] A bench or profile shows the problem and the improvement
- [ ] A bench covers the changed hot-path function
- [ ] No new per-request syscalls, canonicalization, or regex/glob compilation
- [ ] No new steady-state allocations without justification
- [ ] Caches have bounded size and staleness and don't weaken security checks
- [ ] Binary size impact checked for new dependencies
