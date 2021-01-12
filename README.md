# Static Web Server [![Docker Image Version (tag latest semver)](https://img.shields.io/docker/v/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/) [![Build Status](https://travis-ci.com/joseluisq/static-web-server.svg?branch=master)](https://travis-ci.com/joseluisq/static-web-server) [![Docker Image Size (tag)](https://img.shields.io/docker/image-size/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/tags) [![Docker Image](https://img.shields.io/docker/pulls/joseluisq/static-web-server.svg)](https://hub.docker.com/r/joseluisq/static-web-server/)

**STATUS:** This is the WIP v2 release under **active** development.
For stable release v1 and contributions please refer to branch [1.x](https://github.com/joseluisq/static-web-server/tree/1.x).

> A blazing fast static files-serving web server. :zap:

**Static Web Server** is a very small and fast production-ready web server to serving static web files or assets.

## Features

- Built with [Rust](https://rust-lang.org) which is focused on [safety, speed, and concurrency](https://kornel.ski/rust-c-speed).
- Memory safety and very reduced CPU and RAM overhead.
- Blazing fast static files-serving powered by [Warp](https://github.com/seanmonstar/warp/) `v0.2` ([Hyper](https://github.com/hyperium/hyper/) `v0.13`), [Tokio](https://github.com/tokio-rs/tokio) `v0.2` and a set of [awesome crates](./Cargo.toml).
- Suitable for lightweight [GNU/Linux Docker containers](https://hub.docker.com/r/joseluisq/static-web-server/tags). It's a fully __5MB__ static binary thanks to [Rust and Musl libc](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html).
- Opt-in GZip, Deflate and Brotli compression for text-based web files only.
- Compression on demand via [Accept-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
- [Cache Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) headers for assets.
- [HEAD](https://tools.ietf.org/html/rfc7231#section-4.3.2) responses.
- Lightweight and configurable logging via [tracing](https://github.com/tokio-rs/tracing) crate.
- [Termination signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling.
- [HTTP/2](https://tools.ietf.org/html/rfc7540) + TLS support.
- Default and custom error pages.
- [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.
- Configurable using CLI arguments or environment variables.
- First-class [Docker](https://docs.docker.com/get-started/overview/) support. [Scratch](https://hub.docker.com/_/scratch) and latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images available.
- MacOs binary support thanks to [Rust Linux / Darwin Builder](https://github.com/joseluisq/rust-linux-darwin-builder).

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in current work by you, as defined in the Apache-2.0 license, shall be dual licensed as described below, without any additional terms or conditions.

Feel free to send some [Pull request](https://github.com/joseluisq/static-web-server/pulls) or [issue](https://github.com/joseluisq/static-web-server/issues).

## License

This work is primarily distributed under the terms of both the [MIT license](LICENSE-MIT) and the [Apache License (Version 2.0)](LICENSE-APACHE).

© 2019-present [Jose Quintana](https://git.io/joseluisq)
