# Static Web Server v2 - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

_**Note:** See changelog for v1 under the [1.x](https://github.com/joseluisq/static-web-server/blob/1.x/CHANGELOG.md) branch._

## v2.0.0-beta.7 - 2021-07-09

Seventh and last beta release `v2.0.0-beta.7` with notable changes.

__Updates__

- [9e90b38](https://github.com/joseluisq/static-web-server/commit/9e90b38) Hyper 0.14.10 dependency update which fixes two [security issues](https://github.com/hyperium/hyper/releases/tag/v0.14.10).

__Features__

- [432b591](https://github.com/joseluisq/static-web-server/commit/432b591) FreeBSD i686 and x86_64 targets support.
  - i686-unknown-freebsd
  - x86_64-unknown-freebsd

Find the binaries for new targets attached to this release.

__Performance__

- [70a76ed](https://github.com/joseluisq/static-web-server/commit/70a76ed) Optimize root path of static file module which increases performance and reduces memory usage.

__Codebase__

- [2aa130d](https://github.com/joseluisq/static-web-server/commit/2aa130d) Move source `./public` to `./docker` dir.

## v2.0.0-beta.6 - 2021-07-07

Sixth beta release `v2.0.0-beta.6` with notable changes.

__Updates__

- [33040d0](https://github.com/joseluisq/static-web-server/commit/33040d0) Update dependencies including latest Tokio and related crates (also [a4ef322](https://github.com/joseluisq/static-web-server/commit/a4ef322), [26b3fbc](https://github.com/joseluisq/static-web-server/commit/26b3fbc), [e07c333](https://github.com/joseluisq/static-web-server/commit/e07c333)).

__Fixes__

- [a1b7836](https://github.com/joseluisq/static-web-server/commit/a1b7836) Missing `Content-Type` header for directory listing index and error pages.

__Features__

- [e2bf778](https://github.com/joseluisq/static-web-server/commit/e2bf778) Windows 64-bit target support. It also improves Ctrl+C signal handling cross-platform. Note Windows ARM64 is in stand by temporarily, see README file for more details.
- [0fa5015](https://github.com/joseluisq/static-web-server/commit/0fa5015) Windows/Linux i686 targets support and one Windows x86_64
  - i686-pc-windows-msvc
  - i686-unknown-linux-gnu
  - i686-unknown-linux-musl
  - x86_64-pc-windows-gnu
- [59cf8bc](https://github.com/joseluisq/static-web-server/commit/59cf8bc) More text-based mime types for compression.
  - text/csv
  - text/calendar
  - text/markdown
  - text/x-yaml
  - text/x-toml
  - application/rtf
  - application/xhtml+xml

Find the binaries for new targets attached to this release and all targets supported also described in the README file.

__Refactorings__

- [2a699e4](https://github.com/joseluisq/static-web-server/commit/2a699e4) Follow symlinks during directory listing, displaying the index page properly for symlinks that points to directories or files.
- [b4f1bcc](https://github.com/joseluisq/static-web-server/commit/b4f1bcc) Prefer stabilized `Poll::map_err` on compression stream.
- [55ffd06](https://github.com/joseluisq/static-web-server/commit/55ffd06) Handle potencial panic for 404/50x error page responses.
- [920acb2](https://github.com/joseluisq/static-web-server/commit/920acb2) Prefer `to_owned()` for string literals over `to_string()` in some cases.
- [c0dca6e](https://github.com/joseluisq/static-web-server/commit/c0dca6e) Improve directory path scanning when directory listing.
- [0ed6287](https://github.com/joseluisq/static-web-server/commit/0ed6287) Auto compression error result logging.
- [87b8744](https://github.com/joseluisq/static-web-server/commit/87b8744) Minor server config info updates.
- [b025536](https://github.com/joseluisq/static-web-server/commit/b025536) Minor code styling and docs changes.

__Release notes__ 

FreeBSD i686/x86_64 binaries are coming in next and last beta release which is very close to the final v2 releasing.

## v2.0.0-beta.5 - 2021-06-22

Fifth beta release `v2.0.0-beta.5` with notable changes.

__Updates__

- [5343a22](https://github.com/joseluisq/static-web-server/commit/5343a22) Update dependencies including latest Hyper, Tokio and related crates. (also [bcb8493](https://github.com/joseluisq/static-web-server/commit/bcb8493), [e51f969](https://github.com/joseluisq/static-web-server/commit/e51f969), [e51f969](https://github.com/joseluisq/static-web-server/commit/e51f969))

__Features__

- [c96af53](https://github.com/joseluisq/static-web-server/commit/c96af53) Security headers for HTTP/2 by default (`--security-headers`). PR [#44](https://github.com/joseluisq/static-web-server/pull/44) resolves [#39](https://github.com/joseluisq/static-web-server/issues/39)
- [3c95d1a](https://github.com/joseluisq/static-web-server/commit/3c95d1a) Support five more targets. (also [e6faff8](https://github.com/joseluisq/static-web-server/commit/e6faff8))
  - `x86_64-unknown-linux-gnu`
  - `aarch64-apple-darwin`
  - `aarch64-unknown-linux-gnu`
  - `aarch64-unknown-linux-musl`
  - `arm-unknown-linux-gnueabihf`

Find binaries for those targets attached to this release.

__Note about releases__

[Rust Nightly](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) is powering the releases from now on the CI. This makes possible to reach more targets in the future.
For more details about it see [Rust Nightly targets supported](https://doc.rust-lang.org/nightly/rustc/platform-support.html).
However clarify that the `static-web-server` project is not using any nightly feature but only _**stable Rust**_ ones and the project is also tested against nightly and stable Rust on CI periodically in order to be notified in case of _"regressions or bugs introduced in Nightly Rust"_. However it is [known](https://stackoverflow.com/a/56067977/2510591) that the nightly compiler is very stable therefore the reason why we have chosen it for release targets via CI like many other popular Rust projects.
In any case, please don't hesitate to file an issue or send a PR.

__Refactorings__

- [2b2da3a](https://github.com/joseluisq/static-web-server/commit/2b2da3a) `--http2-tls-cert` and `--http2-tls-key` options now require `--http2` enabled.
- [6fe04a5](https://github.com/joseluisq/static-web-server/commit/6fe04a5) Update Docker files in order to get the new Linux binary source.
- [77d231c](https://github.com/joseluisq/static-web-server/commit/77d231c) Drop redudant reference on CORS module.
- [d5189ec](https://github.com/joseluisq/static-web-server/commit/d5189ec) Drop root arc-path on static files module.

## v2.0.0-beta.4 - 2021-06-02

Fourth beta release `v2.0.0-beta.4` with notable changes.

__Updates__

- [a8b9379](https://github.com/joseluisq/static-web-server/commit/a8b9379) Binaries compiled with latest Rust [1.52.1](https://blog.rust-lang.org/2021/05/10/Rust-1.52.1.html) release.
- [c3389cc](https://github.com/joseluisq/static-web-server/commit/c3389cc) Update dependencies including latest Hyper, Tokio and related crates. (also [7cbe483](https://github.com/joseluisq/static-web-server/commit/7cbe483))

__Features__

- [21bdf8c](https://github.com/joseluisq/static-web-server/commit/21bdf8c) Support inheriting TCP listener from parent process via file descriptor (`-f`, `--fd`). PR [#40](https://github.com/joseluisq/static-web-server/pull/40) by [@tim-seoss](https://github.com/tim-seoss).
- [5428eb3](https://github.com/joseluisq/static-web-server/commit/5428eb3) Redefined directory listing (`-z`, `--directory-listing`). PR [#41](https://github.com/joseluisq/static-web-server/pull/41)
- [d389803](https://github.com/joseluisq/static-web-server/commit/d389803) Opt-in response body auto compression (Gzip, Deflate, Brotli) based on `Accept-Encoding` header (`-x`, `--compression`).
- [74b9eaf](https://github.com/joseluisq/static-web-server/commit/74b9eaf) Just one file associated metadata per request as possible.
- [af9a329](https://github.com/joseluisq/static-web-server/commit/af9a329) CORS support (`-c`, ` --cors-allow-origins`).
- [6ed3fe5](https://github.com/joseluisq/static-web-server/commit/6ed3fe5) Unix-like termination signals handling.

__Refactorings__

- [a8d462a](https://github.com/joseluisq/static-web-server/commit/a8d462a) Drop `Warp` in favor of just `Hyper` + `Tokio`. PR [#38](https://github.com/joseluisq/static-web-server/pull/38)
- [04ec1b1](https://github.com/joseluisq/static-web-server/commit/04ec1b1) One worker thread per available core by default (`-n`, `--threads-multiplier`).
- [991d4b8](https://github.com/joseluisq/static-web-server/commit/991d4b8) Introduce a custom Hyper service implementation for the HTTP1 & HTTP2 web servers.
- [58ff9b7](https://github.com/joseluisq/static-web-server/commit/58ff9b7) Reject non `HEAD` or `GET` requests on static assets and error page handlers.
- [5cede7e](https://github.com/joseluisq/static-web-server/commit/5cede7e) Log info for compression and directory listing features.

__Docs__

All feature flags as well as their equivalent environment variables are described on the updated [README](https://github.com/joseluisq/static-web-server#usage) file.

## v2.0.0-beta.3 - 2021-04-20

Third beta release `v2.0.0-beta.3` with notable changes.

__Updates__

- [7f29c90](https://github.com/joseluisq/static-web-server/commit/7f29c90) Binaries compiled with latest Rust [1.51.0](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html) release.
- [97d75e0](https://github.com/joseluisq/static-web-server/commit/97d75e0) [Alpine 3.13](https://alpinelinux.org/posts/Alpine-3.13.0-released.html) Docker image.
- [97d75e0](https://github.com/joseluisq/static-web-server/commit/97d75e0) Update dependencies including latest **Tokio** `v1`,  **Warp** `v0.3` (with **Hyper** `v0.14`) and related crates (also [e9384e9](https://github.com/joseluisq/static-web-server/commit/e9384e9), [5d4421d](https://github.com/joseluisq/static-web-server/commit/5d4421d))

__Refactorings__

- [5d8b266](https://github.com/joseluisq/static-web-server/commit/5d8b266) Static server configuration and static default error pages content.
- [e853410](https://github.com/joseluisq/static-web-server/commit/e853410) Drop support for Deflate compression.
- [bbb5a8f](https://github.com/joseluisq/static-web-server/commit/bbb5a8f) Improve log information on server runtime setup.
- [c05471f](https://github.com/joseluisq/static-web-server/commit/c05471f) Tokio server tasks simplifications.
- [7ea40a7](https://github.com/joseluisq/static-web-server/commit/7ea40a7) Minor CLI typos.

__Fixes__

- [99b8b7e](https://github.com/joseluisq/static-web-server/commit/99b8b7e) Linking error for `ring` crate during Darwin build.

__Docs__

- [3c36c9b](https://github.com/joseluisq/static-web-server/commit/3c36c9b) Minor README description improvements.

## v2.0.0-beta.2 - 2021-01-30

Second beta release `v2.0.0-beta.2` with notable changes.

__Updates__

- [9867d71](https://github.com/joseluisq/static-web-server/commit/9867d71) Update dependencies including latest **Tokio** `v1`,  **Warp** `v0.3` (with **Hyper** `v0.14`) and related crates. (also [a4421c6](https://github.com/joseluisq/static-web-server/commit/a4421c6), [960a681](https://github.com/joseluisq/static-web-server/commit/960a681), [960a681](https://github.com/joseluisq/static-web-server/commit/960a681))

__Features__

- [3007e74](https://github.com/joseluisq/static-web-server/commit/3007e74) **Project sponsor support.** Consider to support the project via [github.com/sponsors/joseluisq](https://github.com/sponsors/joseluisq) or [paypal.me/joseluisqs](https://paypal.me/joseluisqs).
- [360ae99](https://github.com/joseluisq/static-web-server/commit/360ae99) Worker threads multiplier option `--threads-multiplier` which provides the ability to customize number of worker threads.
- [ed0d6ac](https://github.com/joseluisq/static-web-server/commit/ed0d6ac) Custom error pages support.
- [4667b10](https://github.com/joseluisq/static-web-server/commit/4667b10) HTTP/2 + TLS support.
- [8c4ce94](https://github.com/joseluisq/static-web-server/commit/8c4ce94) CORS support.

More details about features on [README](https://github.com/joseluisq/static-web-server/) file.

__Refactorings__

- [6d3e2d1](https://github.com/joseluisq/static-web-server/commit/6d3e2d1) Remove redundant `'static` lifetime on constants.
- [866c7cd](https://github.com/joseluisq/static-web-server/commit/866c7cd) Remove Tokio `macros` feature.
- [f7f2bf6](https://github.com/joseluisq/static-web-server/commit/f7f2bf6) Some improvement suggestions by `Clippy`.
- [bff49a0](https://github.com/joseluisq/static-web-server/commit/bff49a0) Few improvement on filter and helper modules.

__Codebase__

- [7265f6b](https://github.com/joseluisq/static-web-server/commit/7265f6b) Github Actions as new CI.
- [c63b549](https://github.com/joseluisq/static-web-server/commit/c63b549) Remove Travis CI.
- [65250c0](https://github.com/joseluisq/static-web-server/commit/65250c0) Minor simplications on server module.
- [b94fe72](https://github.com/joseluisq/static-web-server/commit/b94fe72) Update core modules structure.
- [da5bdc3](https://github.com/joseluisq/static-web-server/commit/da5bdc3) Re-export few core lib modules.
- [57c27f4](https://github.com/joseluisq/static-web-server/commit/57c27f4) Deny(warnings) on lib
- [a3744d4](https://github.com/joseluisq/static-web-server/commit/a3744d4) Simplify conditionals on rejection filter.

__Docs__

- [933a3c4](https://github.com/joseluisq/static-web-server/commit/933a3c4) Feature documentations updates (also [0ef21c4](https://github.com/joseluisq/static-web-server/commit/0ef21c4))
- [78033d0](https://github.com/joseluisq/static-web-server/commit/78033d0) CLI arguments and environment variables descriptions.

## v2.0.0-beta.1 - 2021-01-12

First major beta release `v2.0.0-beta.1` with notable changes.

#### Built-in features

It uses **Tokio** `v0.2` and **Warp** `v0.2` (**Hyper** `v0.13`).
PR [#28](https://github.com/joseluisq/static-web-server/pull/28)

- Environment variables and CLI arguments setup
- Lightweight and configurable logging
- Head responses support
- GZip, Deflate and Brotli compression support
- Compression for text-based web file only
- Termination signal handling.
- Default error pages (404, 500x, etc)
- GZip, Deflate or Brotli compression can be optional
- Compression on demand via `Accept-Encoding` header
- Cache control headers for assets
- Docker scratch & Alpine images
- MacOs binary support

#### Bugfixes

- Resolves [#24](https://github.com/joseluisq/static-web-server/issues/24) - Error on 8GB file

#### Additional features

- Resolves [#17](https://github.com/joseluisq/static-web-server/issues/17) - Make assets directory path optional. Since this major release doesn't include an assets dir just a root.
