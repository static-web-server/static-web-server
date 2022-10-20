# Static Web Server v2 - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

_**Note:** See changelog for v1 under the [1.x](https://github.com/static-web-server/static-web-server/blob/1.x/CHANGELOG.md) branch._

## v2.13.1 - 2022-10-17

__Fixes__

- [41dd5c6](https://github.com/static-web-server/static-web-server/commit/41dd5c6) Bugfix/security dependency updates including cxx and proc-macro2 crates.
- [abef785](https://github.com/static-web-server/static-web-server/commit/abef785) Directory listing JSON syntax error when requesting for an empty folder.

__Refactorings__

- [d1b72fd](https://github.com/static-web-server/static-web-server/commit/d1b72fd) Significant `~58%` performance boost for serving static files and `~10%` *(CPU)* / `~52%` *(RAM)* less resource utilization. PR [#153](https://github.com/static-web-server/static-web-server/issues/153) resolves [#146](https://github.com/static-web-server/static-web-server/issues/146) reported by [@jonashaag](https://github.com/jonashaag).

## v2.13.0 - 2022-10-12

__Fixes__

- [cce7a85](https://github.com/static-web-server/static-web-server/commit/cce7a85) Bugfix/security dependency updates including jemallocator, http headers, async-compression, rustls-pemfile, tracing and other crates (also [ed8dec3](https://github.com/static-web-server/static-web-server/commit/ed8dec3), [ea0facd](https://github.com/static-web-server/static-web-server/commit/ea0facd)).
- [3c863fd](https://github.com/static-web-server/static-web-server/commit/3c863fd) Directory listing links not encoded properly. PR [#150](https://github.com/static-web-server/static-web-server/issues/150) fixes [#149](https://github.com/static-web-server/static-web-server/issues/149) reported by [@nestor-custodio](https://github.com/nestor-custodio).

__Features__

- [f369c80](https://github.com/static-web-server/static-web-server/commit/f369c80) CORS exposed headers support via new `--cors-expose-headers` option. PR [#144](https://github.com/static-web-server/static-web-server/pull/144) by [@nelsonjchen](https://github.com/nelsonjchen). See [docs](https://sws.joseluisq.net/features/cors/#exposed-headers).
- [997e493](https://github.com/static-web-server/static-web-server/commit/997e493) HTML/JSON support for directory listing entries via new `--directory-listing-format` option. PR [#151](https://github.com/static-web-server/static-web-server/pull/151). See [docs](https://sws.joseluisq.net/features/directory-listing/#output-format).

__Refactorings__

- [61d4bb2](https://github.com/static-web-server/static-web-server/commit/61d4bb2) Restore ANSI terminal colors for Unix systems previously disabled.

__Docs__

- [3d8c74b](https://github.com/static-web-server/static-web-server/commit/3d8c74b) Directory listing format description.

__Codebase__
 
- [6a5ed83](https://github.com/static-web-server/static-web-server/commit/6a5ed83) Update CI workflow dependencies.

## v2.12.0 - 2022-09-27

__Fixes__

- [467affc](https://github.com/static-web-server/static-web-server/commit/467affc) Bugfix/security dependency updates including tokio, serde, tracing, h2, httparse, futures and other crates (also [303d1b4](https://github.com/static-web-server/static-web-server/commit/303d1b4), [c340f8f](https://github.com/static-web-server/static-web-server/commit/c340f8f)).
- [82caf15](https://github.com/static-web-server/static-web-server/commit/82caf15) Potential segfault in the `time` crate. `RUSTSEC-2020-0071` [#142](https://github.com/static-web-server/static-web-server/issues/142)
- [76fd7ea](https://github.com/static-web-server/static-web-server/commit/76fd7ea) Unmaintained `ansi_term` crate. `RUSTSEC-2021-0139` [#143](https://github.com/static-web-server/static-web-server/issues/143)

__Features__

- [91b6ba2](https://github.com/static-web-server/static-web-server/commit/91b6ba2) Relative paths for directory listing entries. PR [#137](https://github.com/static-web-server/static-web-server/pull/137) resolves [#136](https://github.com/static-web-server/static-web-server/issues/136) suggested by [@jtackaberry](https://github.com/jtackaberry). See [docs](https://sws.joseluisq.net/features/directory-listing/#relative-paths-for-entries).
- [5f10771](https://github.com/static-web-server/static-web-server/commit/5f10771) Log Real Remote IP in case of proxies. PR [#138](https://github.com/static-web-server/static-web-server/pull/138) by [@dlvoy](https://github.com/dlvoy). See [docs](https://sws.joseluisq.net/features/logging/#log-real-remote-ip).
- [48f9458](https://github.com/static-web-server/static-web-server/commit/48f9458) Support for serving pre-compressed (Gzip/Brotli) files. PR [#139](https://github.com/static-web-server/static-web-server/pull/139) resolves [#114](https://github.com/static-web-server/static-web-server/issues/114) suggested by [@JonasGilg](https://github.com/JonasGilg). See [docs](https://sws.joseluisq.net/features/compression-static/).

__Refactorings__

- [e9a4aa3](https://github.com/static-web-server/static-web-server/commit/e9a4aa3) Directory listing module.
- [eee45f9](https://github.com/static-web-server/static-web-server/commit/eee45f9) Remove indirections on static file module (performance improvement).

__Docs__

- [59a75e6](https://github.com/static-web-server/static-web-server/commit/59a75e6) Fix documentation typos. PR [#140](https://github.com/static-web-server/static-web-server/pull/140) by [@kianmeng](https://github.com/kianmeng).
- [3ca743a](https://github.com/static-web-server/static-web-server/commit/3ca743a) Page for pre-compressed files serving feature. See [docs](https://sws.joseluisq.net/features/compression-static/).
- [88a886f](https://github.com/static-web-server/static-web-server/commit/88a886f) Building project from source now requires Rust `1.59.0` or later. See [docs](https://sws.joseluisq.net/building-from-source/#building-project-from-source).

__Codebase__
 
- [5148da9](https://github.com/static-web-server/static-web-server/commit/5148da9) CI workflow for Rust security audit.
- [28f8818](https://github.com/static-web-server/static-web-server/commit/28f8818) CI development job for typos. PR [#141](https://github.com/static-web-server/static-web-server/pull/141) by [@kianmeng](https://github.com/kianmeng). See [docs](https://sws.joseluisq.net/features/logging/#log-real-remote-ip).

## v2.11.0 - 2022-08-15

__Fixes__

- [1b7636c](https://github.com/static-web-server/static-web-server/commit/1b7636c) Bugfix/security dependency updates including tokio, serde, tracing, libc, futures and other crates (also [6840d0f](https://github.com/static-web-server/static-web-server/commit/6840d0f), [32517b6](https://github.com/static-web-server/static-web-server/commit/32517b6)).
- [6570498](https://github.com/static-web-server/static-web-server/commit/6570498) Enable the missing `windows-service` option when used via the config file.

__Features__

- [5163564](https://github.com/static-web-server/static-web-server/commit/5163564) New `redirect-trailing-slash` option. PR [#131](https://github.com/static-web-server/static-web-server/pull/131) by [@phartenfeller](https://github.com/phartenfeller). See [docs](https://sws.joseluisq.net/features/trailing-slash-redirect/).

__Docs__

- [10f403f](https://github.com/static-web-server/static-web-server/commit/10f403f) Redirect trailing slash page.
- [e4228af](https://github.com/static-web-server/static-web-server/commit/e4228af) Typos and content improvements (also [e674940](https://github.com/static-web-server/static-web-server/commit/e674940)).

## v2.10.0 - 2022-07-10

__Fixes__

- [b902cb7](https://github.com/static-web-server/static-web-server/commit/b902cb7) Bugfix/security dependency updates including tokio, hyper, tracing, jemallocator and other crates (also [5c9b797](https://github.com/static-web-server/static-web-server/commit/5c9b797), [4cf9a6b](https://github.com/static-web-server/static-web-server/commit/4cf9a6b)).
- [b73959f](https://github.com/static-web-server/static-web-server/commit/b73959f) Fix wrong prefix config file path (`\\?\`) when logging on Windows.

__Features__

- [5163564](https://github.com/static-web-server/static-web-server/commit/5163564) URL Rewrites with pattern matching support. PR [#122](https://github.com/static-web-server/static-web-server/pull/122). See [docs](https://sws.joseluisq.net/features/url-rewrites/).
- [5ef3b62](https://github.com/static-web-server/static-web-server/commit/5ef3b62) URL Redirects with pattern matching. PR [#123](https://github.com/static-web-server/static-web-server/pull/123). See [docs](https://sws.joseluisq.net/features/url-rewrites/).
- [9072977](https://github.com/static-web-server/static-web-server/commit/9072977) Homebrew installation support for MacOS/Linux. See [docs](https://sws.joseluisq.net/download-and-install/#macos).
- [975132f](https://github.com/static-web-server/static-web-server/commit/975132f) [Scoop](https://scoop.sh/#/apps?q=static-web-server&s=0&d=1&o=true) installation support for Windows. See [docs](https://sws.joseluisq.net/download-and-install/#windows).
- [78a5611](https://github.com/static-web-server/static-web-server/commit/78a5611) Alpine 3.16 Docker image.

__Docs__

- [b0ca3d1](https://github.com/static-web-server/static-web-server/commit/b0ca3d1) Several doc typo fixes.

## v2.9.0 - 2022-05-28

__Fixes__

- [446576a](https://github.com/static-web-server/static-web-server/commit/446576a) Bugfix/security dependency updates including tokio, hyper, rustls, compression, windows-rs, serde, log and other crates (also [fa531a0](https://github.com/static-web-server/static-web-server/commit/fa531a0), [0879c84](https://github.com/static-web-server/static-web-server/commit/0879c84)).

__Features__

- [3d1776d](https://github.com/static-web-server/static-web-server/commit/3d1776d) Windows Service support via new `--windows-service` option. PR [#110](https://github.com/static-web-server/static-web-server/pull/110) resolves [#65](https://github.com/static-web-server/static-web-server/issues/65) suggested by [@bubnenkoff](https://github.com/bubnenkoff). See [docs](https://sws.joseluisq.net/features/windows-service/).
- [bd78034](https://github.com/static-web-server/static-web-server/commit/bd78034) Include request URI on tracing log for 404/50x errors. [#108](https://github.com/static-web-server/static-web-server/issues/108) suggested by [@stappersg](https://github.com/stappersg).
- [b49395a](https://github.com/static-web-server/static-web-server/commit/b49395a) Log request file with its remote address (IP) via new `--log-remote-address` option. PR [#112](https://github.com/static-web-server/static-web-server/pull/112) resolves [#111](https://github.com/static-web-server/static-web-server/issues/111) suggested by [@nicheath](https://github.com/nicheath). See [docs](https://sws.joseluisq.net/features/logging/#log-remote-addresses).

__Docs__

- [a793b58](https://github.com/static-web-server/static-web-server/commit/a793b58) Improve basic auth feature page. See [docs](https://sws.joseluisq.net/features/basic-authentication/).
- [ae0dcfd](https://github.com/static-web-server/static-web-server/commit/ae0dcfd) Windows Service feature page. See [docs](https://sws.joseluisq.net/features/windows-service/).
- [2d71de6](https://github.com/static-web-server/static-web-server/commit/2d71de6) Log remote address feature. See [docs](https://sws.joseluisq.net/features/logging/#log-remote-addresses).

## v2.8.0 - 2022-05-04

__Fixes__

- [446576a](https://github.com/static-web-server/static-web-server/commit/446576a) Bugfix/security dependency updates including http, tokio, httparse, windows-rs, serde, log and other crates.

__Features__

- [1fd3e48](https://github.com/static-web-server/static-web-server/commit/1fd3e48) Configuration file support. PR [#101](https://github.com/static-web-server/static-web-server/pull/101). See [docs](https://sws.joseluisq.net/configuration/config-file/).
- [62ebe52](https://github.com/static-web-server/static-web-server/commit/62ebe52) Custom HTTP headers via config file. See [docs](https://sws.joseluisq.net/features/custom-http-headers/).

__Refactorings__

- [9f4bbd7](https://github.com/static-web-server/static-web-server/commit/9f4bbd7) Update `tokio-rustls` to `v0.23`.
- [024531c](https://github.com/static-web-server/static-web-server/commit/024531c) Move to maintained jemallocator (`tikv-jemallocator`) on Linux (Musl libc).
- [3e40153](https://github.com/static-web-server/static-web-server/commit/3e40153) Remove deprecated `git.io` link. PR [#103](https://github.com/static-web-server/static-web-server/pull/103) by [@renbaoshuo](https://github.com/renbaoshuo).
- [959c325](https://github.com/static-web-server/static-web-server/commit/959c325) `PathBuf` data type for cli/file config path options.

__Docs__

- [7dda2ea](https://github.com/static-web-server/static-web-server/commit/7dda2ea) Config file and custom http headers.

## v2.7.1 - 2022-04-17

__Fixes__

- [9c58496](https://github.com/static-web-server/static-web-server/commit/9c58496) Bugfix/security dependency updates including httparse, flate2, h2, tracing, brotli, windows-rs and other crates (also [bc62634](https://github.com/static-web-server/static-web-server/commit/bc62634), [8a1d1cb](https://github.com/static-web-server/static-web-server/commit/8a1d1cb), [eabc559](https://github.com/static-web-server/static-web-server/commit/eabc559)).
- [041f0f8](https://github.com/static-web-server/static-web-server/commit/041f0f8) Prevent arbitrary files access on Windows.<br>
  It mitigates accessing files outside of server root directory on Windows when a driver label is used as part of a request URL.<br>
  E.g `http://localhost:1234/whatever/c:/windows/win.ini`.

__Refactorings__

- [fa05773](https://github.com/static-web-server/static-web-server/commit/fa05773) Small performance improvement for tracing.

__Docs__

- [9c58496](https://github.com/static-web-server/static-web-server/commit/9c58496) Build documentation from source.

## v2.7.0 - 2022-03-21

__Fixes__

- [dc8bc4d](https://github.com/static-web-server/static-web-server/commit/dc8bc4d) Bugfix/security dependency updates including h2, tracing, listenfd, mio, libc, syn and other crates (also [3b2a287](https://github.com/static-web-server/static-web-server/commit/3b2a287), [d57ee2f](https://github.com/static-web-server/static-web-server/commit/d57ee2f), [15cf1ac](https://github.com/static-web-server/static-web-server/commit/15cf1ac)).
- [da85b16](https://github.com/static-web-server/static-web-server/commit/da85b16) `--cors-allow-origins` doesn't assign headers properly. PR [#87](https://github.com/static-web-server/static-web-server/pull/87) resolves [#86](https://github.com/static-web-server/static-web-server/issues/86) reported by [@mr-moon](https://github.com/mr-moon).
- [dcc8a32](https://github.com/static-web-server/static-web-server/commit/dcc8a32) Security Alpine `3.15` Docker image upgrade. PR [#92](https://github.com/static-web-server/static-web-server/pull/92).

__Features__

- [da85b16](https://github.com/static-web-server/static-web-server/commit/da85b16) CORS allowed headers support via the new `-j, --cors-allow-headers` flags. PR [#87](https://github.com/static-web-server/static-web-server/pull/87). See [docs](https://sws.joseluisq.net/features/cors/#allowed-headers).
- [da85b16](https://github.com/static-web-server/static-web-server/commit/da85b16) Support for HTTP `OPTIONS` method requests. PR [#87](https://github.com/static-web-server/static-web-server/pull/87). See [docs](https://sws.joseluisq.net/features/http-methods/).
- [6204205](https://github.com/static-web-server/static-web-server/commit/6204205) `Cache-Control` for AVIF and JPEG XL mime types. PR [#88](https://github.com/static-web-server/static-web-server/pull/88) by [@csmith](https://github.com/csmith). See [docs](https://sws.joseluisq.net/features/cache-control-headers/#one-year).
- [cba4a83](https://github.com/static-web-server/static-web-server/commit/cba4a83) Fallback page option via the new `--page-fallback` flag. PR [#91](https://github.com/static-web-server/static-web-server/pull/91) by [@firstdorsal](https://github.com/firstdorsal). See [docs](https://sws.joseluisq.net/features/error-pages/#fallback-page-for-use-with-client-routers).

__Refactorings__

- [d33d093](https://github.com/static-web-server/static-web-server/commit/d33d093) Reduce few allocations on HTTP request handler.
- [06cc379](https://github.com/static-web-server/static-web-server/commit/06cc379) Reduce small allocation when encoding headers during compression.
- [a5e87e5](https://github.com/static-web-server/static-web-server/commit/a5e87e5) Typed `Content-Type` header for error pages and dir listing responses.

__Docs__

- [781ba91](https://github.com/static-web-server/static-web-server/commit/781ba91) CORS allowed headers. See [docs](https://sws.joseluisq.net/features/cors/#allowed-headers).
- [0957a11](https://github.com/static-web-server/static-web-server/commit/0957a11) HTTP methods section. See [docs](https://sws.joseluisq.net/features/http-methods/).

__Testing__

- [7b6fc0b](https://github.com/static-web-server/static-web-server/commit/7b6fc0b) `Cache-Control` test cases.
- [f22b952](https://github.com/static-web-server/static-web-server/commit/f22b952) Stable Rust for ARM CI pipelines.

## v2.6.0 - 2022-02-28

__Fixes__

- [fb84c0b](https://github.com/static-web-server/static-web-server/commit/fb84c0b) Bugfix/security dependency updates including hyper, tokio, httparse, futures, tracing, headers and other crates (also [7f70a13](https://github.com/static-web-server/static-web-server/commit/7f70a13), [d3fb137](https://github.com/static-web-server/static-web-server/commit/d3fb137)).

__Features__

- [7d32a67](https://github.com/static-web-server/static-web-server/commit/7d32a67) Multi-arch Docker images (Scratch/Alpine). PR [#82](https://github.com/static-web-server/static-web-server/pull/82) resolves [#54](https://github.com/static-web-server/static-web-server/issues/54).
  - New `armv7-unknown-linux-musleabihf` (armv7) and `arm-unknown-linux-musleabihf` (armv6) binary targets.
  - New Docker images for `linux/arm64`, `linux/386`, `linux/arm/v7` and `linux/arm/v6` platforms.
- [50974fe](https://github.com/static-web-server/static-web-server/commit/50974fe) Compress WebAssembly (`application/wasm`) files. PR [#84](https://github.com/static-web-server/static-web-server/pull/84) by [@acelot](https://github.com/acelot). See [docs](https://sws.joseluisq.net/features/compression/).
- [70ec60c](https://github.com/static-web-server/static-web-server/commit/70ec60c) Arch Linux [AUR package](https://aur.archlinux.org/packages/static-web-server-bin) support. See [docs](https://sws.joseluisq.net/download-and-install/).

__Refactorings__

- [e109b77](https://github.com/static-web-server/static-web-server/commit/e109b77) Improve startup server error messages providing context.
- [c085147](https://github.com/static-web-server/static-web-server/commit/c085147) Prefer `cfg(unix)` instead of `cfg(not(windows))`.

__Docs__

- [eb482a4](https://github.com/static-web-server/static-web-server/commit/eb482a4) Documentation for Multi-arch Docker images. See [docs](https://sws.joseluisq.net/features/docker/).
- [70ec60c](https://github.com/static-web-server/static-web-server/commit/70ec60c) Documentation for Arch Linux support. See [docs](https://sws.joseluisq.net/download-and-install/).

## v2.6.0-beta.2 - 2022-02-08

__Fixes__

- [65007f9](https://github.com/static-web-server/static-web-server/commit/65007f9) Wrong binary path for alpine docker image.

## v2.6.0-beta.1 - 2022-02-08

__Fixes__

- [fb84c0b](https://github.com/static-web-server/static-web-server/commit/fb84c0b) Bugfix dependency updates including httparse, futures and other crates.

__Features__

- [c2ae6a5](https://github.com/static-web-server/static-web-server/commit/c2ae6a5) Multi-arch Docker images. PR [#82](https://github.com/static-web-server/static-web-server/pull/82) resolves [#54](https://github.com/static-web-server/static-web-server/issues/54).
  - New `armv7-unknown-linux-musleabihf` (armv7) and `arm-unknown-linux-musleabihf` (armv6) binary targets.
  - New Docker images for `linux/arm64`, `linux/386`, `linux/arm/v7` and `linux/arm/v6` platforms.

## v2.5.0 - 2022-02-04

__Fixes__

- [3df07aa](https://github.com/static-web-server/static-web-server/commit/3df07aa) Bugfix dependency updates including Tokio, libc , h2, tracing, brotli other crates. (also [5f9f9f9](https://github.com/static-web-server/static-web-server/commit/5f9f9f9), [3df07aa](https://github.com/static-web-server/static-web-server/commit/3df07aa), [0c1a6c1](https://github.com/static-web-server/static-web-server/commit/0c1a6c1)).

__Features__

- [3224261](https://github.com/static-web-server/static-web-server/commit/3224261) Configurable grace period support after a `SIGTERM`. PR [#80](https://github.com/static-web-server/static-web-server/pull/80) resolves [#79](https://github.com/static-web-server/static-web-server/issues/79) suggested by [@jtackaberry](https://github.com/jtackaberry). See [docs](https://sws.joseluisq.net/features/graceful-shutdown/#graceful-shutdown) for more details.

__Refactorings__

- [4caf0aa](https://github.com/static-web-server/static-web-server/commit/4caf0aa) Log `info` entry after `ctrl+c` on Windows.

## v2.4.0 - 2022-01-06

__Fixes__

- [fd227b3](https://github.com/static-web-server/static-web-server/commit/fd227b3) Bugfix dependency updates including Tokio, futures, http, syn, libc and other crates. (also [fd227b3](https://github.com/static-web-server/static-web-server/commit/fd227b3), [7becd4e](https://github.com/static-web-server/static-web-server/commit/7becd4e)).

__Refactorings__

- [5926c9b](https://github.com/static-web-server/static-web-server/commit/5926c9b) Trailing slash checking and redirection for directory requests. PR [#74](https://github.com/static-web-server/static-web-server/pull/74) resolves [#73](https://github.com/static-web-server/static-web-server/issues/73) suggested by [@knyzorg](https://github.com/knyzorg).

__Features__

- [ac8f87c](https://github.com/static-web-server/static-web-server/commit/ac8f87c) Alpine 3.14 Docker images.

## v2.3.0 - 2021-12-11

__Fixes__

- [366e6a9](https://github.com/static-web-server/static-web-server/commit/366e6a9) Security/bug fixes dependency updates including Hyper, Tokio, h2, libc, futures and other crates. (Also [dfe87c7](https://github.com/static-web-server/static-web-server/commit/dfe87c7), [1231b50](https://github.com/static-web-server/static-web-server/commit/1231b50)).

__Features__

- [688d1b2](https://github.com/static-web-server/static-web-server/commit/688d1b2) Opt-in sorting by `Name`, `Last Modified` and `File Size` in ascending/descending order via the new `--directory-listing-order` option. More details on [directory listing documentation](https://sws.joseluisq.net/examples/directory-listing/#sorting). PR [#71](https://github.com/static-web-server/static-web-server/pull/71) resolves [#68](https://github.com/static-web-server/static-web-server/issues/68) suggested by [@igoro00](https://github.com/igoro00).

## v2.2.0 - 2021-11-04

__Fixes__

- [c264f2f](https://github.com/static-web-server/static-web-server/commit/c264f2f) Update dependencies (also [e127a1f](https://github.com/static-web-server/static-web-server/commit/e127a1f)).

__Features__

- [0a02da3](https://github.com/static-web-server/static-web-server/commit/0a02da3) [Graceful Shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) support for HTTP/1 - HTTP/2 servers by default. PR [#62](https://github.com/static-web-server/static-web-server/pull/62) resolves [#61](https://github.com/static-web-server/static-web-server/issues/53) suggested by [@pdfrod](https://github.com/pdfrod).

__Refactorings__

- [6f10ef1](https://github.com/static-web-server/static-web-server/commit/6f10ef1) Disable ANSI for tracing logs on Windows in order to display characters correctly.
- [17ceec0](https://github.com/static-web-server/static-web-server/commit/17ceec0) Log Basic Authentication info.

__Docs__

- [b501c40](https://github.com/static-web-server/static-web-server/commit/b501c40) Project Website - [sws.joseluisq.net](https://sws.joseluisq.net). PR [#56](https://github.com/static-web-server/static-web-server/pull/56).

## v2.1.0 - 2021-10-23

__Fixes__

* [5f3842b](https://github.com/static-web-server/static-web-server/commit/5f3842b) Update dependencies including Hyper, Tokio, h2, futures, tracing bug/security fixes and related crates (also [5528bcb](https://github.com/static-web-server/static-web-server/commit/5528bcb), [dc98fbb](https://github.com/static-web-server/static-web-server/commit/dc98fbb)).
* [62e98c6](https://github.com/static-web-server/static-web-server/commit/62e98c6) `aarch64-unknown-linux-musl` build fails using Rust nightly.

__Features__

- [abc76a8](https://github.com/static-web-server/static-web-server/commit/abc76a8) Basic HTTP Authentication support via the new `--basic-auth` option. PR [#55](https://github.com/static-web-server/static-web-server/pull/55) resolves [#53](https://github.com/static-web-server/static-web-server/issues/53) suggested by [@bjornharrtell](https://github.com/bjornharrtell).

__Refactorings__

- [0273611](https://github.com/static-web-server/static-web-server/commit/0273611) Prefer `futures-util` over `futures` dependency.
- [c3bfa68](https://github.com/static-web-server/static-web-server/commit/c3bfa68) Use [Rust 1.56.0 (2021 Edition)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html) on CI.

__Docs__

- [f89c5c9](https://github.com/static-web-server/static-web-server/commit/f89c5c9) Describe Basic HTTP Authentication feature.
- [a6d0e53](https://github.com/static-web-server/static-web-server/commit/a6d0e53) Minor general description improvements.

## v2.0.3 - 2021-09-29

__Fixes__

* [5de5874](https://github.com/static-web-server/static-web-server/commit/5de5874) Update dependencies including Hyper, Tokio, http, futures, tracing bug fixes and related crates (also [1c6c873](https://github.com/static-web-server/static-web-server/commit/1c6c873) [34efa49](https://github.com/static-web-server/static-web-server/commit/34efa49), [6fb832b](https://github.com/static-web-server/static-web-server/commit/6fb832b))

__Refactorings__

- [b2f09ab](https://github.com/static-web-server/static-web-server/commit/b2f09ab) Remove optional extra Docker volume `/public` of `scratch` and `alpine` images. PR [#52](https://github.com/static-web-server/static-web-server/pull/52) resolves [#51](https://github.com/static-web-server/static-web-server/issues/51) reported by [@bergi9](https://github.com/bergi9).
- [906106f](https://github.com/static-web-server/static-web-server/commit/906106f) Remove never read `origins_str` field on Cors module.
- [6f7a6bc](https://github.com/static-web-server/static-web-server/commit/6f7a6bc) Fix Rust edition idioms.

__Tests__

* [166e869](https://github.com/static-web-server/static-web-server/commit/166e869) More Cors test cases.

## v2.0.2 - 2021-08-29

__Fixes__

* [ab83e2a](https://github.com/static-web-server/static-web-server/commit/ab83e2a) Update dependencies including Hyper, h2, httparse bug fixes. Tokio leak fix and related crates (also [adb8ca6](https://github.com/static-web-server/static-web-server/commit/adb8ca6))

## v2.0.1 - 2021-08-18

__Fixes__

* [2459ec4](https://github.com/static-web-server/static-web-server/commit/2459ec4) Return incorrect first bytes range when final bytes are requested. For example a request using `Range: bytes=-10` header returned incorrectly the first 10 bytes rather than the last 10 ones.

__Updates__

* [122e1bd](https://github.com/static-web-server/static-web-server/commit/122e1bd) Update dependencies including Hyper and Tokio bug fixes, Brotli, Serde and related crates (also [0b413f9](https://github.com/static-web-server/static-web-server/commit/0b413f9), [fa130fa](https://github.com/static-web-server/static-web-server/commit/fa130fa), [167e1de](https://github.com/static-web-server/static-web-server/commit/167e1de), [fa32375](https://github.com/static-web-server/static-web-server/commit/fa32375), [2d1c5f3](https://github.com/static-web-server/static-web-server/commit/2d1c5f3))

__Refactorings__

* [8fc1812](https://github.com/static-web-server/static-web-server/commit/8fc1812) Remove needless borrow on static file and handle modules.
* [27f5687](https://github.com/static-web-server/static-web-server/commit/27f5687) UTF-8 for default Docker `index.html` file.
* [ffb2e54](https://github.com/static-web-server/static-web-server/commit/ffb2e54) Update Rust stable to 1.54.0 on CI.

__Tests__

* [5cdcffc](https://github.com/static-web-server/static-web-server/commit/5cdcffc) More directory listing test cases.
* [c7e8ec9](https://github.com/static-web-server/static-web-server/commit/c7e8ec9) More static files test cases.
* [37f2371](https://github.com/static-web-server/static-web-server/commit/37f2371) More static file methods and compression test cases.
* [dd7f995](https://github.com/static-web-server/static-web-server/commit/dd7f995) More static file test cases for during request handling.

__Docs__

* [f389cbc](https://github.com/static-web-server/static-web-server/commit/f389cbc) Minor badges and link updates.

## v2.0.0 - 2021-07-18

The second major stable release is finally available after around half a year of work. It introduces notable changes including new features, performance improvements and new targets support like ARM64 and OSes like FreeBSD.

This version was re-written almost from scratch on top of [Hyper](https://github.com/hyperium/hyper) and [Tokio](https://github.com/tokio-rs/tokio) runtime which give us the [Rust asynchronous ability](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html) by default and latest HTTP/1 - HTTP/2 implementation improvements.
However it still try to keep the same principles of its first version: lightness and easy to use. Therefore a migration should not be a big deal.

Your feedback is very appreciated.

### Features

This list only shows new features not present in previous v1.

- Static Web Server is now asynchronous by default powered by latest Hyper and Tokio.
- It supports opt-in GZip, Deflate and Brotli compression for text-based web files only.
- HTTP/2 + TLS support (via `--http2` option).
- [Security headers](https://github.com/static-web-server/static-web-server/pull/44) like STS, CSP and others for HTTP/2 by default.
- Customizable number of worker threads (via `--threads-multiplier` option).
- [Redesigned directory listing](https://github.com/static-web-server/static-web-server/pull/41) (via `--directory-listing` option).
- Cache control header is now optional (via `--cache-control-headers`).
- Ability to accept a socket listener as a file descriptor for use in sandboxing and on-demand applications (E.g [systemd](http://0pointer.de/blog/projects/socket-activation.html)). Via `--fd` option. Thanks to [@tim-seoss](https://github.com/tim-seoss).
- Binaries for various i686/x86_64 targets (Linux/Windows/FreeBSD) and ARM/ARM64 (Linux/Macos M1)

For the full list, options details and usage please check it out the [README](./README.md) file.

### Breaking changes

This major version has few breaking changes.
However a migration should not represent a problem. Please have in mind the following changes:

- The server supports now only a root directory path (via `--root` or its equivalent env) so an assets path option is no longer required.
- Cache control headers is applied to assets in an arbitrary manner. See [src/control_headers.rs](src/control_headers.rs) for more details.
- OpenSSL TLS for HTTP/1 is not longer supported instead for the HTTP/2 + TLS (via `--http2` option) the server uses [h2](https://github.com/hyperium/h2) which is on top of [Rustls](https://github.com/ctz/rustls). It means that instead of using a .p12 or .pfx file you can now use only a certificate file along with its private key. More details on [README](./README.md) file.

The rest of known options are equivalent to v1 except the new ones of course.
However it's worth to always recommend to test this server update first with your applications on a development environment or similar.

Please see the full list of options with their details on the [README](./README.md) file.

---

Changes after the latest `v2.0.0-beta.7` release:

__Performance__

- [157ade1](https://github.com/static-web-server/static-web-server/commit/157ade1) Decrease few allocations during 404/50x error page responses.
- [941572c](https://github.com/static-web-server/static-web-server/commit/941572c) Reduce few allocations on control headers checking.

__Features__

- [012b626](https://github.com/static-web-server/static-web-server/commit/012b626) Cache control headers optional via `--cache-control-headers`.

__Refactorings__

- [5aa587f](https://github.com/static-web-server/static-web-server/commit/5aa587f) Minor syntax improvements on static file module.
- [45988db](https://github.com/static-web-server/static-web-server/commit/45988db) Minor style updates on server module.

__Docs__

- [5bcc629](https://github.com/static-web-server/static-web-server/commit/5bcc629) FreeBSD targets description.
- [dffdf5c](https://github.com/static-web-server/static-web-server/commit/dffdf5c) Changelog support.

## v2.0.0-beta.7 - 2021-07-09

Seventh and last beta release `v2.0.0-beta.7` with notable changes.

__Updates__

- [9e90b38](https://github.com/static-web-server/static-web-server/commit/9e90b38) Hyper 0.14.10 dependency update which fixes two [security issues](https://github.com/hyperium/hyper/releases/tag/v0.14.10).

__Features__

- [432b591](https://github.com/static-web-server/static-web-server/commit/432b591) FreeBSD i686 and x86_64 targets support.
  - i686-unknown-freebsd
  - x86_64-unknown-freebsd

Find the binaries for new targets attached to this release.

__Performance__

- [70a76ed](https://github.com/static-web-server/static-web-server/commit/70a76ed) Optimize root path of static file module which increases performance and reduces memory usage.

__Codebase__

- [2aa130d](https://github.com/static-web-server/static-web-server/commit/2aa130d) Move source `./public` to `./docker` dir.

## v2.0.0-beta.6 - 2021-07-07

Sixth beta release `v2.0.0-beta.6` with notable changes.

__Updates__

- [33040d0](https://github.com/static-web-server/static-web-server/commit/33040d0) Update dependencies including latest Tokio and related crates (also [a4ef322](https://github.com/static-web-server/static-web-server/commit/a4ef322), [26b3fbc](https://github.com/static-web-server/static-web-server/commit/26b3fbc), [e07c333](https://github.com/static-web-server/static-web-server/commit/e07c333)).

__Fixes__

- [a1b7836](https://github.com/static-web-server/static-web-server/commit/a1b7836) Missing `Content-Type` header for directory listing index and error pages.

__Features__

- [e2bf778](https://github.com/static-web-server/static-web-server/commit/e2bf778) Windows 64-bit target support. It also improves Ctrl+C signal handling cross-platform. Note Windows ARM64 is in stand by temporarily, see README file for more details.
- [0fa5015](https://github.com/static-web-server/static-web-server/commit/0fa5015) Windows/Linux i686 targets support and one Windows x86_64
  - i686-pc-windows-msvc
  - i686-unknown-linux-gnu
  - i686-unknown-linux-musl
  - x86_64-pc-windows-gnu
- [59cf8bc](https://github.com/static-web-server/static-web-server/commit/59cf8bc) More text-based mime types for compression.
  - text/csv
  - text/calendar
  - text/markdown
  - text/x-yaml
  - text/x-toml
  - application/rtf
  - application/xhtml+xml

Find the binaries for new targets attached to this release and all targets supported also described in the README file.

__Refactorings__

- [2a699e4](https://github.com/static-web-server/static-web-server/commit/2a699e4) Follow symlinks during directory listing, displaying the index page properly for symlinks that points to directories or files.
- [b4f1bcc](https://github.com/static-web-server/static-web-server/commit/b4f1bcc) Prefer stabilized `Poll::map_err` on compression stream.
- [55ffd06](https://github.com/static-web-server/static-web-server/commit/55ffd06) Handle potential panic for 404/50x error page responses.
- [920acb2](https://github.com/static-web-server/static-web-server/commit/920acb2) Prefer `to_owned()` for string literals over `to_string()` in some cases.
- [c0dca6e](https://github.com/static-web-server/static-web-server/commit/c0dca6e) Improve directory path scanning when directory listing.
- [0ed6287](https://github.com/static-web-server/static-web-server/commit/0ed6287) Auto compression error result logging.
- [87b8744](https://github.com/static-web-server/static-web-server/commit/87b8744) Minor server config info updates.
- [b025536](https://github.com/static-web-server/static-web-server/commit/b025536) Minor code styling and docs changes.

__Release notes__ 

FreeBSD i686/x86_64 binaries are coming in next and last beta release which is very close to the final v2 releasing.

## v2.0.0-beta.5 - 2021-06-22

Fifth beta release `v2.0.0-beta.5` with notable changes.

__Updates__

- [5343a22](https://github.com/static-web-server/static-web-server/commit/5343a22) Update dependencies including latest Hyper, Tokio and related crates. (also [bcb8493](https://github.com/static-web-server/static-web-server/commit/bcb8493), [e51f969](https://github.com/static-web-server/static-web-server/commit/e51f969), [e51f969](https://github.com/static-web-server/static-web-server/commit/e51f969))

__Features__

- [c96af53](https://github.com/static-web-server/static-web-server/commit/c96af53) Security headers for HTTP/2 by default (`--security-headers`). PR [#44](https://github.com/static-web-server/static-web-server/pull/44) resolves [#39](https://github.com/static-web-server/static-web-server/issues/39)
- [3c95d1a](https://github.com/static-web-server/static-web-server/commit/3c95d1a) Support five more targets. (also [e6faff8](https://github.com/static-web-server/static-web-server/commit/e6faff8))
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

- [2b2da3a](https://github.com/static-web-server/static-web-server/commit/2b2da3a) `--http2-tls-cert` and `--http2-tls-key` options now require `--http2` enabled.
- [6fe04a5](https://github.com/static-web-server/static-web-server/commit/6fe04a5) Update Docker files in order to get the new Linux binary source.
- [77d231c](https://github.com/static-web-server/static-web-server/commit/77d231c) Drop redundant reference on CORS module.
- [d5189ec](https://github.com/static-web-server/static-web-server/commit/d5189ec) Drop root arc-path on static files module.

## v2.0.0-beta.4 - 2021-06-02

Fourth beta release `v2.0.0-beta.4` with notable changes.

__Updates__

- [a8b9379](https://github.com/static-web-server/static-web-server/commit/a8b9379) Binaries compiled with latest Rust [1.52.1](https://blog.rust-lang.org/2021/05/10/Rust-1.52.1.html) release.
- [c3389cc](https://github.com/static-web-server/static-web-server/commit/c3389cc) Update dependencies including latest Hyper, Tokio and related crates. (also [7cbe483](https://github.com/static-web-server/static-web-server/commit/7cbe483))

__Features__

- [21bdf8c](https://github.com/static-web-server/static-web-server/commit/21bdf8c) Support inheriting TCP listener from parent process via file descriptor (`-f`, `--fd`). PR [#40](https://github.com/static-web-server/static-web-server/pull/40) by [@tim-seoss](https://github.com/tim-seoss).
- [5428eb3](https://github.com/static-web-server/static-web-server/commit/5428eb3) Redefined directory listing (`-z`, `--directory-listing`). PR [#41](https://github.com/static-web-server/static-web-server/pull/41)
- [d389803](https://github.com/static-web-server/static-web-server/commit/d389803) Opt-in response body auto compression (Gzip, Deflate, Brotli) based on `Accept-Encoding` header (`-x`, `--compression`).
- [74b9eaf](https://github.com/static-web-server/static-web-server/commit/74b9eaf) Just one file associated metadata per request as possible.
- [af9a329](https://github.com/static-web-server/static-web-server/commit/af9a329) CORS support (`-c`, ` --cors-allow-origins`).
- [6ed3fe5](https://github.com/static-web-server/static-web-server/commit/6ed3fe5) Unix-like termination signals handling.

__Refactorings__

- [a8d462a](https://github.com/static-web-server/static-web-server/commit/a8d462a) Drop `Warp` in favor of just `Hyper` + `Tokio`. PR [#38](https://github.com/static-web-server/static-web-server/pull/38)
- [04ec1b1](https://github.com/static-web-server/static-web-server/commit/04ec1b1) One worker thread per available core by default (`-n`, `--threads-multiplier`).
- [991d4b8](https://github.com/static-web-server/static-web-server/commit/991d4b8) Introduce a custom Hyper service implementation for the HTTP1 & HTTP2 web servers.
- [58ff9b7](https://github.com/static-web-server/static-web-server/commit/58ff9b7) Reject non `HEAD` or `GET` requests on static assets and error page handlers.
- [5cede7e](https://github.com/static-web-server/static-web-server/commit/5cede7e) Log info for compression and directory listing features.

__Docs__

All feature flags as well as their equivalent environment variables are described on the updated [README](https://github.com/static-web-server/static-web-server#usage) file.

## v2.0.0-beta.3 - 2021-04-20

Third beta release `v2.0.0-beta.3` with notable changes.

__Updates__

- [7f29c90](https://github.com/static-web-server/static-web-server/commit/7f29c90) Binaries compiled with latest Rust [1.51.0](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html) release.
- [97d75e0](https://github.com/static-web-server/static-web-server/commit/97d75e0) [Alpine 3.13](https://alpinelinux.org/posts/Alpine-3.13.0-released.html) Docker image.
- [97d75e0](https://github.com/static-web-server/static-web-server/commit/97d75e0) Update dependencies including latest **Tokio** `v1`,  **Warp** `v0.3` (with **Hyper** `v0.14`) and related crates (also [e9384e9](https://github.com/static-web-server/static-web-server/commit/e9384e9), [5d4421d](https://github.com/static-web-server/static-web-server/commit/5d4421d))

__Refactorings__

- [5d8b266](https://github.com/static-web-server/static-web-server/commit/5d8b266) Static server configuration and static default error pages content.
- [e853410](https://github.com/static-web-server/static-web-server/commit/e853410) Drop support for Deflate compression.
- [bbb5a8f](https://github.com/static-web-server/static-web-server/commit/bbb5a8f) Improve log information on server runtime setup.
- [c05471f](https://github.com/static-web-server/static-web-server/commit/c05471f) Tokio server tasks simplifications.
- [7ea40a7](https://github.com/static-web-server/static-web-server/commit/7ea40a7) Minor CLI typos.

__Fixes__

- [99b8b7e](https://github.com/static-web-server/static-web-server/commit/99b8b7e) Linking error for `ring` crate during Darwin build.

__Docs__

- [3c36c9b](https://github.com/static-web-server/static-web-server/commit/3c36c9b) Minor README description improvements.

## v2.0.0-beta.2 - 2021-01-30

Second beta release `v2.0.0-beta.2` with notable changes.

__Updates__

- [9867d71](https://github.com/static-web-server/static-web-server/commit/9867d71) Update dependencies including latest **Tokio** `v1`,  **Warp** `v0.3` (with **Hyper** `v0.14`) and related crates. (also [a4421c6](https://github.com/static-web-server/static-web-server/commit/a4421c6), [960a681](https://github.com/static-web-server/static-web-server/commit/960a681), [960a681](https://github.com/static-web-server/static-web-server/commit/960a681))

__Features__

- [3007e74](https://github.com/static-web-server/static-web-server/commit/3007e74) **Project sponsor support.** Consider to support the project via [github.com/sponsors/joseluisq](https://github.com/sponsors/joseluisq) or [paypal.me/joseluisqs](https://paypal.me/joseluisqs).
- [360ae99](https://github.com/static-web-server/static-web-server/commit/360ae99) Worker threads multiplier option `--threads-multiplier` which provides the ability to customize number of worker threads.
- [ed0d6ac](https://github.com/static-web-server/static-web-server/commit/ed0d6ac) Custom error pages support.
- [4667b10](https://github.com/static-web-server/static-web-server/commit/4667b10) HTTP/2 + TLS support.
- [8c4ce94](https://github.com/static-web-server/static-web-server/commit/8c4ce94) CORS support.

More details about features on [README](https://github.com/static-web-server/static-web-server/) file.

__Refactorings__

- [6d3e2d1](https://github.com/static-web-server/static-web-server/commit/6d3e2d1) Remove redundant `'static` lifetime on constants.
- [866c7cd](https://github.com/static-web-server/static-web-server/commit/866c7cd) Remove Tokio `macros` feature.
- [f7f2bf6](https://github.com/static-web-server/static-web-server/commit/f7f2bf6) Some improvement suggestions by `Clippy`.
- [bff49a0](https://github.com/static-web-server/static-web-server/commit/bff49a0) Few improvement on filter and helper modules.

__Codebase__

- [7265f6b](https://github.com/static-web-server/static-web-server/commit/7265f6b) Github Actions as new CI.
- [c63b549](https://github.com/static-web-server/static-web-server/commit/c63b549) Remove Travis CI.
- [65250c0](https://github.com/static-web-server/static-web-server/commit/65250c0) Minor simplications on server module.
- [b94fe72](https://github.com/static-web-server/static-web-server/commit/b94fe72) Update core modules structure.
- [da5bdc3](https://github.com/static-web-server/static-web-server/commit/da5bdc3) Re-export few core lib modules.
- [57c27f4](https://github.com/static-web-server/static-web-server/commit/57c27f4) Deny(warnings) on lib
- [a3744d4](https://github.com/static-web-server/static-web-server/commit/a3744d4) Simplify conditionals on rejection filter.

__Docs__

- [933a3c4](https://github.com/static-web-server/static-web-server/commit/933a3c4) Feature documentations updates (also [0ef21c4](https://github.com/static-web-server/static-web-server/commit/0ef21c4))
- [78033d0](https://github.com/static-web-server/static-web-server/commit/78033d0) CLI arguments and environment variables descriptions.

## v2.0.0-beta.1 - 2021-01-12

First major beta release `v2.0.0-beta.1` with notable changes.

#### Built-in features

It uses **Tokio** `v0.2` and **Warp** `v0.2` (**Hyper** `v0.13`).
PR [#28](https://github.com/static-web-server/static-web-server/pull/28)

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

- Resolves [#24](https://github.com/static-web-server/static-web-server/issues/24) - Error on 8GB file

#### Additional features

- Resolves [#17](https://github.com/static-web-server/static-web-server/issues/17) - Make assets directory path optional. Since this major release doesn't include an assets dir just a root.
