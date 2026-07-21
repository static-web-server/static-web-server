<div>
  <div align="center">
    <a href="https://static-web-server.net" title="static-web-server website">
      <img src="https://static-web-server.net/assets/sws.svg" height="100" width="100"
    /></a>
  </div>

<h1 align="center">Static Web Server</h1>

<h4 align="center">
    A cross-platform, high-performance and asynchronous web server for static file-serving ⚡
  </h4>

<div align="center">
    <a href="https://github.com/static-web-server/static-web-server/actions/workflows/devel.yml" title="devel ci"><img src="https://github.com/static-web-server/static-web-server/actions/workflows/devel.yml/badge.svg?branch=master"></a> 
    <a href="https://hub.docker.com/r/joseluisq/static-web-server/" title="Docker Image Version (tag latest semver)"><img src="https://img.shields.io/docker/v/joseluisq/static-web-server/latest"></a> 
    <a href="https://hub.docker.com/r/joseluisq/static-web-server/tags" title="Docker Image Size (tag)"><img src="https://img.shields.io/docker/image-size/joseluisq/static-web-server/latest"></a> 
    <a href="https://hub.docker.com/r/joseluisq/static-web-server/" title="Docker Image"><img src="https://img.shields.io/docker/pulls/joseluisq/static-web-server.svg"></a> 
    <a href="https://crates.io/crates/static-web-server" title="static-web-server crate"><img src="https://img.shields.io/crates/v/static-web-server.svg"></a> 
    <a href="https://docs.rs/static-web-server" title="static-web-server crate docs"><img src="https://img.shields.io/docsrs/static-web-server/latest?label=docs.rs"></a> 
    <a href="https://github.com/static-web-server/static-web-server/blob/master/LICENSE-APACHE" title="static-web-server license"><img src="https://img.shields.io/crates/l/static-web-server"></a> 
    <a href="https://discord.gg/VWvtZeWAA7" title="Static Web Server Community on Discord">
      <img src="https://img.shields.io/discord/1086203405225164842?logo=discord&label=discord">
    </a>
  </div>
</div>

## Overview

> [!NOTE]
> This is the upcoming **v3**, currently in development. For production use, see **[v2 (LTS)](https://github.com/static-web-server/static-web-server/tree/2.x)**.

**Static Web Server** (**SWS**) is a small, fast, production-ready web server for static file-serving that prioritizes simplicity and ease of use while delivering [high performance and safety](https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html), powered by [Rust](https://rust-lang.org). Built on [Hyper](https://github.com/hyperium/hyper) and [Tokio](https://github.com/tokio-rs/tokio) to deliver efficient and performant [asynchronous I/O](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html) and HTTP/1-2 support with a low resource footprint.

SWS is available for a variety of platforms like `Linux`, `macOS`, `Windows`, `FreeBSD`, `NetBSD`, `Android` and `Docker`.

![static-web-server running](https://github.com/static-web-server/static-web-server/assets/1700322/102bef12-1f30-4054-a1bc-30c650d4ffa7)

## Features

- Built with [Rust](https://rust-lang.org), [memory-safe, fast, and concurrent](https://kornel.ski/rust-c-speed) with low CPU and RAM overhead.
- Single __~4 MB__ static binary with zero runtime dependencies ([Musl libc](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)). Runs on [any Linux distro](https://en.wikipedia.org/wiki/Linux_distribution) or [Docker container](https://hub.docker.com/r/joseluisq/static-web-server/tags).
- Fast, asynchronous static file-serving built on [Hyper](https://github.com/hyperium/hyper/) and [Tokio](https://github.com/tokio-rs/tokio).
- On-the-fly compression (Gzip, Deflate, Brotli, Zstandard) for text-based files via [`Accept-Encoding`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding).
- Serving of [pre-compressed files](https://static-web-server.net/features/compression/) (`.br`, `.gz`, `.zst`) directly from disk.
- Markdown content negotiation to serve `.md` files to clients that [accept](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept) `text/markdown`.
- [Byte-range serving](https://en.wikipedia.org/wiki/Byte_serving) for large file delivery.
- Optional [Cache-Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) headers and ETag with sensible defaults per file type.
- [HTTP/2](https://tools.ietf.org/html/rfc7540) + TLS with automatic [security headers](https://web.dev/security-headers/).
- [Graceful shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) with configurable grace period.
- Structured JSON logging (`--log-format`) and file-based log output (`--log-file`).
- Configurable thread pool (worker and blocking threads).
- Optional [directory listing](https://static-web-server.net/features/directory-listing/) with HTML and JSON output, plus `.tar.gz` download.
- [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) with preflight support.
- [Basic HTTP authentication](https://static-web-server.net/features/basic-authentication/) (BCrypt).
- Custom response headers per file via glob patterns.
- [URL rewrites and redirects](https://static-web-server.net/features/rewrites-and-redirects/) with placeholder replacement.
- [Virtual hosting](https://static-web-server.net/features/virtual-hosting/) with per-host root directories.
- Fallback page for 404 errors (SPA support).
- Custom error pages.
- Built-in HTTP to HTTPS redirect.
- [Health check](https://static-web-server.net/features/health-endpoint/) endpoint (GET/HEAD).
- [Prometheus metrics](https://static-web-server.net/features/metrics/) endpoint with request counts, latency histograms, and connection tracking.
- [Maintenance mode](https://static-web-server.net/features/maintenance-mode/) with configurable status.
- Configuration via CLI arguments, environment variables, or a TOML file.
- [Windows Service](<https://docs.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc783643(v=ws.10)>) support.
- Socket activation ([systemd](http://0pointer.de/blog/projects/socket-activation.html)) for sandboxed and on-demand deployments.
- Available as a [Rust library](https://docs.rs/static-web-server) with opt-in features.
- First-class [Docker](https://docs.docker.com/get-started/overview/) images: [Scratch](https://hub.docker.com/_/scratch), [Alpine Linux](https://hub.docker.com/_/alpine), and [Debian](https://hub.docker.com/_/debian).
- Cross-platform pre-compiled binaries for `Linux`, `macOS`, `Windows`, `FreeBSD`, `NetBSD`, `Android` (`x86_64`, `ARM64`).

## Documentation

Please refer to [The Documentation Website](https://static-web-server.net/) for more details about the API, usage and examples.

## Releases

- [Docker Images](https://hub.docker.com/r/joseluisq/static-web-server/)
- [Release Binaries](https://github.com/static-web-server/static-web-server/releases)
- [Platforms/Architectures Supported](https://static-web-server.net/platforms-architectures/)

## Contributions

Unless you explicitly state otherwise, any contribution you intentionally submitted for inclusion in current work, as defined in the Apache-2.0 license, shall be dual licensed as described below, without any additional terms or conditions.

Feel free to submit a [pull request](https://github.com/static-web-server/static-web-server/pulls) or file an [issue](https://github.com/static-web-server/static-web-server/issues).

## Community

[SWS Community on Discord](https://discord.gg/VWvtZeWAA7)

## License

This work is primarily distributed under the terms of both the [MIT license](LICENSE-MIT) and the [Apache License (Version 2.0)](LICENSE-APACHE).

© 2019-present [Jose Quintana](https://joseluisq.net)
