<div>
  <div align="center">
    <img
      src="https://camo.githubusercontent.com/a08032a2db94aea229991af8f73c45cc95174c8066dc7a6b1f88a79c94cf1093/68747470733a2f2f75706c6f61642e77696b696d656469612e6f72672f77696b6970656469612f636f6d6d6f6e732f7468756d622f642f64352f527573745f70726f6772616d6d696e675f6c616e67756167655f626c61636b5f6c6f676f2e7376672f3130323470782d527573745f70726f6772616d6d696e675f6c616e67756167655f626c61636b5f6c6f676f2e7376672e706e67"
      height="100" width="100"
    />
  </div>

  <h1 align="center">Static Web Server</h1>

  <h4 align="center">
    A blazing fast and asynchronous web server for static files-serving ⚡
  </h4>

<div align="center">
<a href="https://github.com/joseluisq/static-web-server/actions/workflows/devel.yml" title="devel ci"><img src="https://github.com/joseluisq/static-web-server/actions/workflows/devel.yml/badge.svg?branch=master"></a> 
<a href="https://hub.docker.com/r/joseluisq/static-web-server/" title="Docker Image Version (tag latest semver)"><img src="https://img.shields.io/docker/v/joseluisq/static-web-server/2"></a> 
<a href="https://hub.docker.com/r/joseluisq/static-web-server/tags" title="Docker Image Size (tag)"><img src="https://img.shields.io/docker/image-size/joseluisq/static-web-server/2"></a> 
<a href="https://hub.docker.com/r/joseluisq/static-web-server/" title="Docker Image"><img src="https://img.shields.io/docker/pulls/joseluisq/static-web-server.svg"></a> 
<a href="https://sws.joseluisq.net" title="Documentation"><img src="https://img.shields.io/badge/docs-latest-green"></a>
</div>
</div>

## Overview

**Static Web Server** (or **`SWS`** abbreviated) is a very small and fast production-ready web server suitable to serve static web files or assets.

It is focused on **lightness and easy to use** principles but keeping [high performance and safety](https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html) powered by [The Rust Programming Language](https://rust-lang.org).

Written on top of [Hyper](https://github.com/hyperium/hyper) and [Tokio](https://github.com/tokio-rs/tokio) runtime. It provides [concurrent and asynchronous networking abilities](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html) as well as the latest HTTP/1 - HTTP/2 implementations.

It's cross-platform and available for `Linux`, `macOS`, `Windows` and `FreeBSD` (`x86`/`x86_64`,  `ARM`/`ARM64`) as well as `Docker`.

## Features

- Built with [Rust](https://rust-lang.org) which is focused on [safety, speed and concurrency](https://kornel.ski/rust-c-speed).
- Memory safe and very reduced CPU and RAM overhead.
- Blazing fast static files-serving and asynchronous powered by latest [Hyper](https://github.com/hyperium/hyper/), [Tokio](https://github.com/tokio-rs/tokio) and a set of [awesome crates](https://github.com/joseluisq/static-web-server/blob/master/Cargo.toml).
- Single __4MB__ (uncompressed) and fully static binary with no dependencies ([Musl libc](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)). Suitable for running on [any Linux distro](https://en.wikipedia.org/wiki/Linux_distribution) or [Docker container](https://hub.docker.com/r/joseluisq/static-web-server/tags).
- Optional GZip, Deflate or Brotli compression for text-based web files only.
- Compression on demand via [Accept-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
- [Partial Content Delivery](https://en.wikipedia.org/wiki/Byte_serving) support for byte-serving of large files.
- Optional [Cache Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) headers for assets.
- [Termination signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling with [graceful shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) ability and grace period.
- [HTTP/2](https://tools.ietf.org/html/rfc7540) + TLS support.
- [Security headers](https://web.dev/security-headers/) for HTTP/2 by default.
- [HEAD](https://tools.ietf.org/html/rfc7231#section-4.3.2) responses.
- Lightweight and configurable logging via [tracing](https://github.com/tokio-rs/tracing) crate.
- Customizable number of worker threads.
- Optional directory listing.
- [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.
- Basic HTTP Authentication.
- Default and custom error pages.
- Configurable using CLI arguments or environment variables.
- First-class [Docker](https://docs.docker.com/get-started/overview/) support. [Scratch](https://hub.docker.com/_/scratch) and latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images available.
- Ability to accept a socket listener as a file descriptor for use in sandboxing and on-demand applications (E.g [systemd](http://0pointer.de/blog/projects/socket-activation.html)).
- Cross-platform. Binaries available for Linux, macOS, Windows & FreeBSD x86_64 / ARM.

## Documentation

For more details about the API, usage and examples please have a look at [The Documentation Website](https://sws.joseluisq.net/).

## Releases

- [Docker Images](https://hub.docker.com/r/joseluisq/static-web-server/)
- [Release Binaries](https://github.com/joseluisq/static-web-server/releases)
- [Platforms/Architectures Supported](https://sws.joseluisq.net/platforms-architectures/)

## Notes

- If you're looking for `v1` please go to [1.x](https://github.com/joseluisq/static-web-server/tree/1.x) branch.
- If you want to migrate from `v1` to `v2` please take a look at [Migrating from `v1` to `v2`](https://sws.joseluisq.net/migration/) release.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in current work by you, as defined in the Apache-2.0 license, shall be dual licensed as described below, without any additional terms or conditions.

Feel free to send some [Pull request](https://github.com/joseluisq/static-web-server/pulls) or file an [issue](https://github.com/joseluisq/static-web-server/issues).

## License

This work is primarily distributed under the terms of both the [MIT license](LICENSE-MIT) and the [Apache License (Version 2.0)](LICENSE-APACHE).

© 2019-present [Jose Quintana](https://git.io/joseluisq)
