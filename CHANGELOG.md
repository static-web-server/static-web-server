# Static Web Server v2 - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

_**Note:** See changelog for v1 under the [1.x](https://github.com/static-web-server/static-web-server/blob/1.x/CHANGELOG.md) branch._

## v2.33.1 - 2024-11-02

This new `v2.33.1` release brings several security and bug fixes as well as other minor improvements.

__Fixes__

- [93479ba](https://github.com/static-web-server/static-web-server/commit/93479ba) Bugfix/security dependency updates including hyper, tokio, httparse, rustls, regex, once_cell, flate2, async-compression and other crates. PR [#490](https://github.com/static-web-server/static-web-server/pull/490).

__Refactorings__

- [de8482d](https://github.com/static-web-server/static-web-server/commit/de8482d) Do not set `Last-Modified` header if mtime is Unix epoch. PR [#488](https://github.com/static-web-server/static-web-server/pull/488) by [@akhilles](https://github.com/akhilles).

__Docs__

- [30a6409](https://github.com/static-web-server/static-web-server/commit/30a6409) Minor tweaks to `man-pages-completions.md` page.

## v2.33.0 - 2024-09-17

This new `v2.33.0` release brings several security and bug fixes. New features like experimental in-memory files cache with eviction policy support, new subcomand to generate man pages and shell completions as well as other improvements.

Note that experimental features are subject to change in future releases. Feel free to give it a try and let us know your feedback.

__Fixes__

- [e25b586](https://github.com/static-web-server/static-web-server/commit/e25b586) Bugfix/security dependency updates including tokio, rustls, serde, toml, once_cell, flate2, clap and other crates. PR [#479](https://github.com/static-web-server/static-web-server/pull/479).
- [a3d40b8](https://github.com/static-web-server/static-web-server/commit/a3d40b8) Crate: Issues when building SWS without default features. PR [#480](https://github.com/static-web-server/static-web-server/pull/480).
- [6bb6138](https://github.com/static-web-server/static-web-server/commit/6bb6138) Docker: Update Alpine (`3.18.9`) and Debian (`12.7`) Docker images. PR [#478](https://github.com/static-web-server/static-web-server/pull/478).

__Features__

- [5bdfcd4](https://github.com/static-web-server/static-web-server/commit/5bdfcd4) Advanced: Experimental in-memory files cache with eviction policy support via a new advanced config option. See PR [#328](https://github.com/static-web-server/static-web-server/pull/328) description for usage and details.
- [ec85abd](https://github.com/static-web-server/static-web-server/commit/ec85abd) Crate: Add in-memory files cache to the `experimental` Cargo feature. See PR [#482](https://github.com/static-web-server/static-web-server/pull/482) description for more details.
  - **MSRV update**: Note that due to this change, the SWS's *Minimum Supported Rust Version* is now `1.76.0` when building from source or using it as a library. See [docs](https://static-web-server.net/building-from-source/).
- [d567b4e](https://github.com/static-web-server/static-web-server/commit/d567b4e) CLI: Support for generating man pages and shell completions via new `generate` subcomand. PR [#475](https://github.com/static-web-server/static-web-server/pull/475) by [@jcgruenhage](https://github.com/jcgruenhage). See [docs](https://static-web-server.net/features/man-pages-completions/).

For more details see the [v2.33.0 milestone](https://github.com/static-web-server/static-web-server/milestone/23?closed=1) and the full changelog [v2.32.2...v2.33.0](https://github.com/static-web-server/static-web-server/compare/v2.32.2...v2.33.0).

**Acknowledgments**

Thanks to our new donor [@ramkumarkb](https://github.com/ramkumarkb) for supporting the project.

## v2.32.2 - 2024-08-13

This new `v2.32.2` release brings several security and bug fixes as well as other improvements.

__Fixes__

- [634dd98](https://github.com/static-web-server/static-web-server/commit/634dd98) Bugfix/security dependency updates including tokio, rustls, serde, toml, zstd, clap and other crates. PR [#472](https://github.com/static-web-server/static-web-server/pull/472).
- [a72c7b3](https://github.com/static-web-server/static-web-server/commit/a72c7b3) Wrong `Content-Encoding` when serving a pre-compressed file if `compression` and `compression-static` features are enabled. PR [#471](https://github.com/static-web-server/static-web-server/pull/471) fixes [#470](https://github.com/static-web-server/static-web-server/issues/470) reported by [@davinkevin](https://github.com/davinkevin).
- [dd48972](https://github.com/static-web-server/static-web-server/commit/dd48972) CLI (regression): Boolean flags without explicit values do not work. PR [#468](https://github.com/static-web-server/static-web-server/pull/468) fixes [#467](https://github.com/static-web-server/static-web-server/issues/467) reported by [@stardustman](https://github.com/stardustman).
- [915d040](https://github.com/static-web-server/static-web-server/commit/915d040) Tests: Not able to run tests via Cargo when passing non-SWS arguments. PR [#466](https://github.com/static-web-server/static-web-server/pull/466) fixes [#465](https://github.com/static-web-server/static-web-server/issues/465) reported by [@fpletz](https://github.com/fpletz).

__Refactorings__

- [f228a7a](https://github.com/static-web-server/static-web-server/commit/f228a7a) Misc: Add Make a task for building all development Docker images.

For more details see the [v2.32.2 milestone](https://github.com/static-web-server/static-web-server/milestone/22?closed=1) and the full changelog [v2.32.1...v2.32.2](https://github.com/static-web-server/static-web-server/compare/v2.32.1...v2.32.2).

## v2.32.1 - 2024-07-20

This new `v2.32.1` release brings several security and bug fixes as well as other improvements.

__Fixes__

- [cfa3567](https://github.com/static-web-server/static-web-server/commit/cfa3567) Bugfix/security dependency updates including hyper, tokio, rustls, jemallocator, url, zstd, toml, mime_guess and other crates. PR [#463](https://github.com/static-web-server/static-web-server/pull/463).
- [87ce30d](https://github.com/static-web-server/static-web-server/commit/87ce30d) Docker: Update Debian Docker images to 12.6. PR [#461](https://github.com/static-web-server/static-web-server/pull/461).
- [057239d](https://github.com/static-web-server/static-web-server/commit/057239d) Docker: Update Alpine Docker images to 3.18.7. PR [#459](https://github.com/static-web-server/static-web-server/pull/459).

__Refactorings__

- [b3fad98](https://github.com/static-web-server/static-web-server/commit/b3fad98) CI: Remove deprecated bors and improve devel workflow. PR [#458](https://github.com/static-web-server/static-web-server/pull/458).tokio, 
- [e64076c](https://github.com/static-web-server/static-web-server/commit/e64076c) CI: Improve typos workflow configuration. PR [#456](https://github.com/static-web-server/static-web-server/pull/456) by [@szepeviktor](https://github.com/szepeviktor).
- [4c805d6](https://github.com/static-web-server/static-web-server/commit/4c805d6) Remove some redundant async function signatures. PR [#457](https://github.com/static-web-server/static-web-server/pull/457).

__Docs__

- [25b1b1c](https://github.com/static-web-server/static-web-server/commit/25b1b1c) Improve feature, versioning pages and remove dead links. PR [#460](https://github.com/static-web-server/static-web-server/pull/460).

For more details see the [v2.32.1 milestone](https://github.com/static-web-server/static-web-server/milestone/21?closed=1) and the full changelog [v2.32.0...v2.32.1](https://github.com/static-web-server/static-web-server/compare/v2.32.0...v2.32.1).

## v2.32.0 - 2024-06-18

This new `v2.32.0` release brings several bug fixes and a new option to disable symlinks as well as other improvements. 

__Fixes__

- [cd5fa1b](https://github.com/static-web-server/static-web-server/commit/cd5fa1b) Bugfix/security dependency updates including hyper, tokio, regex, serde, httparse and other crates (also [c227302](https://github.com/static-web-server/static-web-server/commit/c227302), [6713932](https://github.com/static-web-server/static-web-server/commit/6713932)).
- [6031a1b](https://github.com/static-web-server/static-web-server/commit/6031a1b) Incorrect `Content-Encoding` for pre-compressed Zstd file requests. PR [#452](https://github.com/static-web-server/static-web-server/pull/452) fixes [#451](https://github.com/static-web-server/static-web-server/issues/451) reported by [@nomeaning777](https://github.com/nomeaning777).
- [3410365](https://github.com/static-web-server/static-web-server/commit/3410365) Duplicated `Vary` response header for `compression` and `compression-static` features. PR [#453](https://github.com/static-web-server/static-web-server/pull/453).

__Features__

- [eeb88da](https://github.com/static-web-server/static-web-server/commit/eeb88da) Disable symlinks via `--disable-symlinks` option. PR [#454](https://github.com/static-web-server/static-web-server/pull/454). See [docs](https://static-web-server.net/features/disable-symlinks).
- [b291189](https://github.com/static-web-server/static-web-server/commit/b291189) Installer: Custom install version and directory options for binary installer. PR [#449](https://github.com/static-web-server/static-web-server/pull/449) by [@frankli0324](https://github.com/frankli0324). See [docs](https://static-web-server.net/download-and-install/#binary-installer-linuxbsds).

__Docs__

- [a888397](https://github.com/static-web-server/static-web-server/commit/a888397) Improve download and install page. PR [#450](https://github.com/static-web-server/static-web-server/pull/450) (also [91f8ec06](https://github.com/static-web-server/static-web-server/commit/91f8ec06)). See [docs](https://static-web-server.net/download-and-install/).

## v2.31.1 - 2024-05-21

This new `v2.31.1` release fixes an issue when running the SWS Linux ARM64 Musl binary on systems with greater memory page sizes than 4KB and re-enables build support for the legacy Windows 7 dropped by the previous release.

__Fixes__

- [c5f851f](https://github.com/static-web-server/static-web-server/commit/c5f851f) Bugfix/security dependency updates.
- [adaddde](https://github.com/static-web-server/static-web-server/commit/adaddde) Jemalloc unsupported system page size in Linux ARM64 Musl. PR [#446](https://github.com/static-web-server/static-web-server/pull/446).
- [1763623](https://github.com/static-web-server/static-web-server/commit/1763623) Lib: Cargo `publish` issue due to missing `build.rs` file include.

__Refactorings__

- [5f116d](https://github.com/static-web-server/static-web-server/commit/5f116d) Re-enable Windows 7 build support. PR [#446](https://github.com/static-web-server/static-web-server/pull/446).<br>
  The following Windows targets will build now using Rust `1.77.2` rather than the latest stable version, except `aarch64-pc-windows-msvc` (a.k.a. Windows ARM64):
  - `x86_64-pc-windows-msvc`
  - `i686-pc-windows-msvc`
  - `x86_64-pc-windows-gnu`

## v2.31.0 - 2024-05-19

This new `v2.31.0` release brings several bug fixes, features like a new compression level option, logs for virtual-hosts and better accepted-encodings handling for pre-compressed files as well as other improvements.

__Fixes__

- [d22e2dd](https://github.com/static-web-server/static-web-server/commit/d22e2dd) Bugfix/security dependency updates including bytes, serde, toml, async-compression, flate2, brotli and other crates (also [ea96328](https://github.com/static-web-server/static-web-server/commit/ea96328)).
- [1aae13c](https://github.com/static-web-server/static-web-server/commit/1aae13c) Directory Listing: Empty file sizes are displayed incorrectly. PR [#385](https://github.com/static-web-server/static-web-server/pull/385) by [@palant](https://github.com/palant).
- [25c171b](https://github.com/static-web-server/static-web-server/commit/25c171b) Directory Listing: File sizes are wrongly displayed in decimal format. PR [#376](https://github.com/static-web-server/static-web-server/pull/376) by [@miroim](https://github.com/miroim).
- [e7bfaa2](https://github.com/static-web-server/static-web-server/commit/e7bfaa2) Lib: Compile errors if only deflate compression is enabled. PR [#383](https://github.com/static-web-server/static-web-server/pull/383) by [@palant](https://github.com/palant).
- [195f706](https://github.com/static-web-server/static-web-server/commit/195f706) Docker: Unsupported system page size when using Linux ARM64 musl (E.g. Raspberry Pi 5). PR [#443](https://github.com/static-web-server/static-web-server/pull/443).
- [d4046d9](https://github.com/static-web-server/static-web-server/commit/d4046d9) Unexpected `cfg` condition-name `wasm` in Rust nightly. PR [#441](https://github.com/static-web-server/static-web-server/pull/441).
- [1b13a74](https://github.com/static-web-server/static-web-server/commit/1b13a74) Misc: Clippy warnings caused by test code. PR [#382](https://github.com/static-web-server/static-web-server/pull/382) by [@palant](https://github.com/palant).
- [0792606](https://github.com/static-web-server/static-web-server/commit/0792606) CI: Code analysis workflow.
- [936b224](https://github.com/static-web-server/static-web-server/commit/936b224) CI: Prevent running the FreeBSD devel pipeline unnecessarily. 

__Refactorings__

- [85e3da](https://github.com/static-web-server/static-web-server/commit/85e3da) Lib: Make request `body` type generic across modules. PR [#375](https://github.com/static-web-server/static-web-server/pull/375) by [@palant](https://github.com/palant).
- [18f550a](https://github.com/static-web-server/static-web-server/commit/18f550a) Directory Listing: Drop custom type conversion between `SystemTime` and `DateTime`. PR [#384](https://github.com/static-web-server/static-web-server/pull/384) by [@palant](https://github.com/palant).
- [219ec6c](https://github.com/static-web-server/static-web-server/commit/219ec6c) Move Remote and Real IP addresses logging to a new module. PR [#388](https://github.com/static-web-server/static-web-server/pull/388).
- [cfd8bb1](https://github.com/static-web-server/static-web-server/commit/cfd8bb1) Re-organize file system-related modules. PR [#424](https://github.com/static-web-server/static-web-server/pull/424).
- [7cf72e6](https://github.com/static-web-server/static-web-server/commit/7cf72e6) Misc: Several project config and doc file improvements.

__Features__

- [9cbf95b](https://github.com/static-web-server/static-web-server/commit/9cbf95b) Compression level option support. PR [#381](https://github.com/static-web-server/static-web-server/pull/381) by [@palant](https://github.com/palant). See [docs](https://static-web-server.net/features/compression/#compression-level).
- [778477e](https://github.com/static-web-server/static-web-server/commit/778477e) Log virtual-hosts information per request. PR [#442](https://github.com/static-web-server/static-web-server/pull/442).
- [7f59da9](https://github.com/static-web-server/static-web-server/commit/7f59da9) Look for all accepted encodings for pre-compressed files but keeping the quality ordering. PR [#439](https://github.com/static-web-server/static-web-server/pull/439) by [@kobutri](https://github.com/kobutri).
- [d2eaa74](https://github.com/static-web-server/static-web-server/commit/d2eaa74) Benchmark: Add Caddy server to performance benchmarks. PR [#379](https://github.com/static-web-server/static-web-server/pull/379) by [@palant](https://github.com/palant).
- [2faeef5](https://github.com/static-web-server/static-web-server/commit/2faeef5) Show detailed information about the server via `--version` (`-V`) flag. PR [#444](https://github.com/static-web-server/static-web-server/pull/444).

## v2.30.0 - 2024-04-29

This new `v2.30.0` release brings security and dependency updates as well as several bug fixes. Overall performance improvements (directory listing particularly), continuous HTTP load testing benchmarks, project refactorings for increased stability, security and correctness as well as several other improvements.

__Fixes__

- [6683446](https://github.com/static-web-server/static-web-server/commit/6683446) Bugfix/security dependency updates including rustls, tokio, async-compression, regex, chrono, clap and other crates (also [8cfc7ed](https://github.com/static-web-server/static-web-server/commit/8cfc7ed), [a876cd5](https://github.com/static-web-server/static-web-server/commit/a876cd5), [69bfdd4](https://github.com/static-web-server/static-web-server/commit/69bfdd4)).
- [c04357e](https://github.com/static-web-server/static-web-server/commit/c04357e) Missing custom headers for directory requests (trailing slash). PR [#333](https://github.com/static-web-server/static-web-server/pull/333).
- [1c4fad2](https://github.com/static-web-server/static-web-server/commit/1c4fad2) CORS does not work properly when used with Basic Auth. PR [#343](https://github.com/static-web-server/static-web-server/pull/343) by [@ms140569](https://github.com/ms140569).
- [528ed08](https://github.com/static-web-server/static-web-server/commit/528ed08) Accept-Encoding handling does not work correctly if only two compression schemes are available. PR [#361](https://github.com/static-web-server/static-web-server/pull/361) by [@palant](https://github.com/palant).
- [c8e39aa](https://github.com/static-web-server/static-web-server/commit/c8e39aa) Errors due to "unused code" when features are disabled. PR [#368](https://github.com/static-web-server/static-web-server/pull/368) by [@palant](https://github.com/palant).
- [5d66301](https://github.com/static-web-server/static-web-server/commit/5d66301) Unreserved characters are percent-encoded in directory listing links. PR [#371](https://github.com/static-web-server/static-web-server/pull/371).
- [114862a](https://github.com/static-web-server/static-web-server/commit/114862a) Malformed UTF-8 file names are not handled correctly. PR [#374](https://github.com/static-web-server/static-web-server/pull/374) by [@palant](https://github.com/palant).

__Features__

- [012ef11](https://github.com/static-web-server/static-web-server/commit/012ef11) Crate: Display platforms-specific documentation on docs.rs.
- [a197f20](https://github.com/static-web-server/static-web-server/commit/a197f20) CI: Load testing benchmarks comparison for each commit via GitHub Actions. PR [#355](https://github.com/static-web-server/static-web-server/pull/355) by [@palant](https://github.com/palant).

__Refactorings__

- [a451a93](https://github.com/static-web-server/static-web-server/commit/a451a93) Improve performance when serving static files. PR [#334](https://github.com/static-web-server/static-web-server/pull/334).
- [e569a71](https://github.com/static-web-server/static-web-server/commit/e569a71) Reduce some allocations in several modules. PR [#337](https://github.com/static-web-server/static-web-server/pull/337).
- [183102d](https://github.com/static-web-server/static-web-server/commit/183102d) Build error when using specific or no Cargo compression features. PR [#339](https://github.com/static-web-server/static-web-server/pull/339).
- [fe6a2a1](https://github.com/static-web-server/static-web-server/commit/fe6a2a1) Move health endpoint-related code into a separate file. PR [#344](https://github.com/static-web-server/static-web-server/pull/344) by [@palant](https://github.com/palant).
- [cc6784a](https://github.com/static-web-server/static-web-server/commit/cc6784a) Move metrics endpoint-related code into a separate file. PR [#345](https://github.com/static-web-server/static-web-server/pull/345) by [@palant](https://github.com/palant).
- [76531e6](https://github.com/static-web-server/static-web-server/commit/76531e6) Move all of Basic authentication logic into basic_auth module. PR [#346](https://github.com/static-web-server/static-web-server/pull/346) by [@palant](https://github.com/palant).
- [d44e5a1](https://github.com/static-web-server/static-web-server/commit/d44e5a1) Move all redirect handling logic into the redirects module. PR [#348](https://github.com/static-web-server/static-web-server/pull/348) by [@palant](https://github.com/palant).
- [e965933](https://github.com/static-web-server/static-web-server/commit/e965933) Move most of CORS-related code into the cors module. PR [#349](https://github.com/static-web-server/static-web-server/pull/349) by [@palant](https://github.com/palant).
- [1246e37](https://github.com/static-web-server/static-web-server/commit/1246e37) Move most of maintenance mode logic into maintenance_mode module. PR [#350](https://github.com/static-web-server/static-web-server/pull/350) by [@palant](https://github.com/palant).
- [941f692](https://github.com/static-web-server/static-web-server/commit/941f692) Move various code related to header handling to the respective modules. PR [#351](https://github.com/static-web-server/static-web-server/pull/351) by [@palant](https://github.com/palant).
- [a13f496](https://github.com/static-web-server/static-web-server/commit/a13f496) Replaced fork of the headers module by an in-tree handler for the Accept-Encoding header. PR [#354](https://github.com/static-web-server/static-web-server/pull/354) by [@palant](https://github.com/palant).
- [207fa4a](https://github.com/static-web-server/static-web-server/commit/207fa4a) Move all rewrite handling logic into the rewrites module. PR [#353](https://github.com/static-web-server/static-web-server/pull/353) by [@palant](https://github.com/palant).
- [c3c55a4](https://github.com/static-web-server/static-web-server/commit/c3c55a4) Prefer querying available cpus using Rust std. PR [#358](https://github.com/static-web-server/static-web-server/pull/358).
- [ddda871](https://github.com/static-web-server/static-web-server/commit/ddda871) Apply the usual post-processing to error responses for consistency. PR [#359](https://github.com/static-web-server/static-web-server/pull/359) by [@palant](https://github.com/palant).
- [cfd1390](https://github.com/static-web-server/static-web-server/commit/cfd1390) Improve performance of directory listings. PR [#357](https://github.com/static-web-server/static-web-server/pull/357) by [@palant](https://github.com/palant).
- [5a4035f](https://github.com/static-web-server/static-web-server/commit/5a4035f) Improve recognition of text MIME types for compression. PR [#360](https://github.com/static-web-server/static-web-server/pull/360) by [@palant](https://github.com/palant).
- [b66c89e](https://github.com/static-web-server/static-web-server/commit/b66c89e) Move all compression-related code into compression and compression_static modules. PR [#369](https://github.com/static-web-server/static-web-server/pull/369) by [@palant](https://github.com/palant).
- [5b5ea98](https://github.com/static-web-server/static-web-server/commit/5b5ea98) Use maud templates and serde_json for directory listings. PR [#367](https://github.com/static-web-server/static-web-server/pull/367) by [@palant](https://github.com/palant).
- [f311e94](https://github.com/static-web-server/static-web-server/commit/f311e94) Move all fallback page logic into the corresponding module. PR [#372](https://github.com/static-web-server/static-web-server/pull/372) by [@palant](https://github.com/palant).
- [7d61c91](https://github.com/static-web-server/static-web-server/commit/7d61c91) Move directory listing initialization into the corresponding module. PR [#373](https://github.com/static-web-server/static-web-server/pull/373) by [@palant](https://github.com/palant).

__Docs__

- [90b6032](https://github.com/static-web-server/static-web-server/commit/90b6032) Add Exherbo Linux install guide. PR [#331](https://github.com/static-web-server/static-web-server/pull/331) by [@davlgd](https://github.com/davlgd). See [docs](https://static-web-server.net/download-and-install/).
- [f534f00](https://github.com/static-web-server/static-web-server/commit/f534f00) Fix typo in GitHub bug report template. PR [#341](https://github.com/static-web-server/static-web-server/pull/341) by [@palant](https://github.com/palant).

For more details see [v2.30.0 milestone](https://github.com/static-web-server/static-web-server/milestone/17?closed=1) and the full changelog [v2.28.0...v2.30.0](https://github.com/static-web-server/static-web-server/compare/v2.28.0...v2.30.0).

## v2.28.0 - 2024-03-09

This new `v2.28.0` release brings several dependency updates and bug fixes. Cancellation ability to shut down the server gracefully on demand when using the library, Docker examples and Windows Firewall instructions as well as other improvements.

__Fixes__

- [769daf1](https://github.com/static-web-server/static-web-server/commit/769daf1) Bugfix/security dependency updates including mio, ring, http, rustls-pemfile, regex, chrono, clap and other crates (also [a97cc77](https://github.com/static-web-server/static-web-server/commit/a97cc77)).
- [e031a7d](https://github.com/static-web-server/static-web-server/commit/e031a7d) Docker: Debian 12.5 image update.
- [b6444f4](https://github.com/static-web-server/static-web-server/commit/b6444f4) Crate: `TryFrom` imported redundantly in Rust nightly. PR [#318](https://github.com/static-web-server/static-web-server/pull/318) by [@yonas](https://github.com/yonas).

__Features__

- [afd6a87](https://github.com/static-web-server/static-web-server/commit/afd6a87) Crate: Cancellation ability for `server::Server::run_server_on_rt` and `server::Server::run_standalone` functions in Linux/BSDs. PR [#319](https://github.com/static-web-server/static-web-server/pull/319) resolves [#315](https://github.com/orgs/static-web-server/discussions/315) suggested by [@hanaTsuk1](https://github.com/hanaTsuk1).

__Refactorings__

- [a68349c](https://github.com/static-web-server/static-web-server/commit/a68349c) Crate: Add targets to Crate docs metadata.
- [afa8575](https://github.com/static-web-server/static-web-server/commit/afa8575) Misc: Refactor the static files module and delegate functionality to separated files. PR [#329](https://github.com/static-web-server/static-web-server/pull/329).

__Docs__

- [9fb2111](https://github.com/static-web-server/static-web-server/commit/9fb2111) Windows Firewall rule instructions for the Windows service feature. See [docs](https://static-web-server.net/features/windows-service/#windows-firewall).
- [668ecfe](https://github.com/static-web-server/static-web-server/commit/668ecfe) Docker and related examples. It resolves [#323](https://github.com/static-web-server/static-web-server/issues/323) suggested by [@hanscees](https://github.com/hanscees). See [docs](https://static-web-server.net/features/docker/#dockerfile).

## v2.27.0 - 2024-02-13

This new `v2.27.0` release brings a few dependency updates and bug fixes. Two new Cargo feature flags and fixes a regression introduced by the previous release when building SWS from source.

__Fixes__

- [ec93d6c](https://github.com/static-web-server/static-web-server/commit/ec93d6c) Bugfix/security dependency updates including chrono, indexmap, indexmap and other crates.

__Features__

- [1a6caa4](https://github.com/static-web-server/static-web-server/commit/1a6caa4) Crate: New `all` and `experimental` Cargo feature flags. PR [#313](https://github.com/static-web-server/static-web-server/pull/313) also fixes [#312](https://github.com/static-web-server/static-web-server/issues/312) reported by [@mattfbacon](https://github.com/mattfbacon). See [docs](https://static-web-server.net/building-from-source/#cargo-features).

## v2.26.0 - 2024-02-10

This new `v2.26.0` release brings several dependency security updates and bug fixes. Support for `Range` requests out of bounds, experimental Tokio Runtime metrics for Prometheus, new Discord server as well as other improvements.

__Fixes__

- [80af0aa](https://github.com/static-web-server/static-web-server/commit/80af0aa) Bugfix/security dependency updates including tokio, regex, chrono, libc, toml, serde and other crates. Also [1d4f423](https://github.com/static-web-server/static-web-server/commit/1d4f423)
- [5623799](https://github.com/static-web-server/static-web-server/commit/5623799) Docker: Alpine 3.18.6 update.
- [a7dc6ac](https://github.com/static-web-server/static-web-server/commit/a7dc6ac) Docker: linux/s590x and linux/ppc64le images are missing dependencies. PR [#309](https://github.com/static-web-server/static-web-server/pull/309) resolves [#308](https://github.com/static-web-server/static-web-server/issues/308) reported by [@glehmann](https://github.com/glehmann).
  - It is solved by dropping support for the `linux/ppc64le` and `linux/s390x` of the Alpine Scratch images because those binaries provided are not static-linked. Prefer the Debian image variant for those targets instead.

__Features__

- [71dd54f](https://github.com/static-web-server/static-web-server/commit/71dd54f) Support for `Range` requests out of bounds. PR [#306](https://github.com/static-web-server/static-web-server/pull/306) resolves [#295](https://github.com/static-web-server/static-web-server/issues/295) suggested by [@bjornharrtell](https://github.com/bjornharrtell).
- [d4427eb](https://github.com/static-web-server/static-web-server/commit/d4427eb) Experimental Tokio Runtime metrics for Prometheus via the new `--experimental-metrics` option. PR [#306](https://github.com/static-web-server/static-web-server/pull/306) by [@pl4nty](https://github.com/pl4nty).
- [fd15914](https://github.com/static-web-server/static-web-server/commit/fd15914) [SWS on Discord](https://discord.gg/VWvtZeWAA7) link.

__Refactorings__

- [563367c](https://github.com/static-web-server/static-web-server/commit/563367c) Minimum Rust stable version 1.74.0.
- [370d288](https://github.com/static-web-server/static-web-server/commit/370d288) Misc: Base fuzz and micro-benchmark testing for static files module. PR [#310](https://github.com/static-web-server/static-web-server/pull/310).

**Acknowledgments**

Thanks to our new donor [@c0m4r](https://github.com/c0m4r) for supporting the project.

## v2.25.0 - 2024-01-23

This new `v2.25.0` release brings several dependency security updates and bug fixes. An optional `Host` URI support for the URL Redirects feature, a bug fix when capturing a Glob pattern using brace expansion for URL Rewrites/Redirects as well as other improvements.

__Fixes__

- [477ed00](https://github.com/static-web-server/static-web-server/commit/477ed00) Bugfix/security dependency updates including rustls, h2, regex, chrono, libc, async-compression, serde and other crates. Also [32e86aa](https://github.com/static-web-server/static-web-server/commit/32e86aa)
- [42f52e8](https://github.com/static-web-server/static-web-server/commit/42f52e8) Fix wrong Glob brace expansion capture in URL Rewrites/Redirects. PR [#304](https://github.com/static-web-server/static-web-server/pull/304).
- [9f2a4f0](https://github.com/static-web-server/static-web-server/commit/9f2a4f0) Docker: Alpine 3.18.5 update.

__Features__

- [8c6ab53](https://github.com/static-web-server/static-web-server/commit/8c6ab53) Optional `Host` URI support for the URL Redirects feature. PR [#301](https://github.com/static-web-server/static-web-server/pull/301).
  This will allow users for example to perform www to non-www redirects or vice versa. See [docs](https://static-web-server.net/features/url-redirects/#host).

__Refactorings__

- [83e4277](https://github.com/static-web-server/static-web-server/commit/83e4277) Migrate TLS module to use `tokio-rustls` 0.25. PR [#303](https://github.com/static-web-server/static-web-server/pull/303).
- [1bbc703](https://github.com/static-web-server/static-web-server/commit/1bbc703) CI: Cache Rust toolchain and Cargo directories for CI devel workflow. PR [#300](https://github.com/static-web-server/static-web-server/pull/300).
- [67a2403](https://github.com/static-web-server/static-web-server/commit/67a2403) CI: Prefer `cross` precompiled binary on CI devel workflow.
- [ef9876a](https://github.com/static-web-server/static-web-server/commit/ef9876a) CI: Prefer `cross` precompiled binary on CI release workflow.

__Docs__

- [3076d08](https://github.com/static-web-server/static-web-server/commit/3076d08) Optional `Host` uri support for URL Redirects feature. See [docs](https://static-web-server.net/features/url-redirects/#host).
- [dedefc5](https://github.com/static-web-server/static-web-server/commit/dedefc5) Fix a few page typos.

**Acknowledgments**

Thanks to our new donors for supporting the project.

## v2.24.2 - 2023-12-28

This new `v2.24.2` release brings general dependency security updates, bug fixes and improvements.

__Fixes__

- [5554522](https://github.com/static-web-server/static-web-server/commit/5554522) Bugfix/security dependency updates including hyper, tokio, rustls/ring, h2, tracing, regex, toml, futures, serde and other crates.
  - Other commit updates: [16f4afd](https://github.com/static-web-server/static-web-server/commit/16f4afd), [76dc853](https://github.com/static-web-server/static-web-server/commit/76dc853), [12dfb56](https://github.com/static-web-server/static-web-server/commit/12dfb56)
- [8cdb305](https://github.com/static-web-server/static-web-server/commit/8cdb305) Docker: Debian 12.4 update.
- [af203ca](https://github.com/static-web-server/static-web-server/commit/af203ca) Docker: Alpine 3.17.6 update.

__Refactorings__

- [96ec477](https://github.com/static-web-server/static-web-server/commit/96ec477) Consistency when importing several types.
- [ab67bd7](https://github.com/static-web-server/static-web-server/commit/ab67bd7) Misc: GitHub issue and pull request template improvements. PR [#294](https://github.com/static-web-server/static-web-server/pull/294). Also [a9d509e](https://github.com/static-web-server/static-web-server/commit/a9d509e).

## v2.24.1 - 2023-11-15

This new `v2.24.1` release brings dependency security updates and bug fixes. In particular, it fixes an issue when executing the previous Windows ARM64 build and other improvements.

__Fixes__

- [c0c88f1](https://github.com/static-web-server/static-web-server/commit/c0c88f1) Bugfix/security dependency updates including tokio, http, rustls-pemfile, tracing, clap and other crates.
- [f4e9142](https://github.com/static-web-server/static-web-server/commit/f4e9142) Windows ARM64 binary does not execute due to missing DLLs. PR [#290](https://github.com/static-web-server/static-web-server/pull/290).

__Refactorings__

- [990bb7c](https://github.com/static-web-server/static-web-server/commit/990bb7c) Statically link the C runtime for Windows MSVC x86 (32-bit) build. PR [#291](https://github.com/static-web-server/static-web-server/pull/291).

__Docs__

- [e99d989](https://github.com/static-web-server/static-web-server/commit/e99d989) Fix typos in README file. PR [#287](https://github.com/static-web-server/static-web-server/pull/287) by [@dynamite-bud](https://github.com/dynamite-bud) (also [a987e37](https://github.com/static-web-server/static-web-server/commit/a987e37)).

__Misc__

- [3099dba](https://github.com/static-web-server/static-web-server/commit/3099dba) CI: Manual Docker build CI workflow for testing. PR [#286](https://github.com/static-web-server/static-web-server/pull/286).
- [680323c](https://github.com/static-web-server/static-web-server/commit/680323c) CI: Manual release build CI workflow for testing. PR [#288](https://github.com/static-web-server/static-web-server/pull/288).

## v2.24.0 - 2023-11-09

This new `v2.24.0` release brings dependency security updates and bug fixes. It introduces three new targets (PowerPC (PPC64LE), S390x and Windows ARM64). Features like automatic TOML configuration file detection at startup and 404/50x error pages loading at runtime as well as several improvements.

__Fixes__

- [e767938](https://github.com/static-web-server/static-web-server/commit/e767938) Bugfix/security dependency updates including ring, rustls, regex, clap, serde, futures, brotli and other crates (also [b0c0775](https://github.com/static-web-server/static-web-server/commit/b0c0775)).
- [4fa09ab](https://github.com/static-web-server/static-web-server/commit/4fa09ab) CI: `cross` does not build when using `libc` 0.2.149+ on NetBSD.

__Features__

- [e89ce29](https://github.com/static-web-server/static-web-server/commit/e89ce29) Automatic TOML configuration file detection at startup. PR [#281](https://github.com/static-web-server/static-web-server/pull/281). See [docs](https://static-web-server.net/configuration/config-file/).
- [fd4bfd4](https://github.com/static-web-server/static-web-server/commit/fd4bfd4) Linux PowerPC (PPC64LE) and S390x targets (also Docker images). PR [#159](https://github.com/static-web-server/static-web-server/pull/159). See [docs](https://static-web-server.net/platforms-architectures/#powerpc).
- [02c6d3e](https://github.com/static-web-server/static-web-server/commit/02c6d3e) Windows ARM64 target. PR [#283](https://github.com/static-web-server/static-web-server/pull/283). See [docs](https://static-web-server.net/platforms-architectures/#arm64_2).
- [1fa9261](https://github.com/static-web-server/static-web-server/commit/1fa9261) Load 404 and 50x error pages at runtime. PR [#284](https://github.com/static-web-server/static-web-server/pull/284) resolves [#98](https://github.com/static-web-server/static-web-server/issues/98) reported by [@Dexus](https://github.com/Dexus).

__Refactorings__

- [4de9acd](https://github.com/static-web-server/static-web-server/commit/4de9acd) Allowed methods response for `OPTIONS` file requests. PR [#278](https://github.com/static-web-server/static-web-server/pull/278).
- [d06ad0f](https://github.com/static-web-server/static-web-server/commit/d06ad0f) Remove some unused TLS configuration APIs and use defaults directly. PR [#279](https://github.com/static-web-server/static-web-server/pull/279).
- [ab16187](https://github.com/static-web-server/static-web-server/commit/ab16187) Improve the server maintenance mode debug logs. PR [#282](https://github.com/static-web-server/static-web-server/pull/282).

__Docs__

- [2798725](https://github.com/static-web-server/static-web-server/commit/2798725) Linux PowerPC (PPC64LE) and S390x targets information. See [docs](https://static-web-server.net/download-and-install/#powerpc).

## v2.23.0 - 2023-10-15

This new `v2.23.0` release brings several dependency updates and bug fixes. New features like multiple index files and maintenance mode support, more performance and resource optimizations (~15% less memory usage), a bug fix for the directory listing, documentation for using SWS in WebAssembly and TrueNAS SCALE and other improvements.

__Fixes__

- [85ea7c4](https://github.com/static-web-server/static-web-server/commit/85ea7c4) Bugfix/security dependency updates including tokio, regex, clap, async-compression (zstd, flate2), tracing, serde and other crates (also [27cb09d](https://github.com/static-web-server/static-web-server/commit/27cb09d)).
- [7c5df01](https://github.com/static-web-server/static-web-server/commit/7c5df01) Wrong directory type for empty files in JSON directory listing. PR [#271](https://github.com/static-web-server/static-web-server/pull/271) resolves [#270](https://github.com/static-web-server/static-web-server/issues/270) reported by [@carueda](https://github.com/carueda).
- [89d70d0](https://github.com/static-web-server/static-web-server/commit/89d70d0) Docker: Debian 12.2 image update.
- [aeebc6f](https://github.com/static-web-server/static-web-server/commit/aeebc6f) Installer: Installer script breakage. PR [#274](https://github.com/static-web-server/static-web-server/pull/274) resolves [#273](https://github.com/static-web-server/static-web-server/issues/273) reported by [@kzhui125](https://github.com/kzhui125).
- [e3cd810](https://github.com/static-web-server/static-web-server/commit/e3cd810) Crate: Docs links in compression module.

__Features__

- [efb2c0c](https://github.com/static-web-server/static-web-server/commit/efb2c0c) Multiple index files support. PR [#267](https://github.com/static-web-server/static-web-server/pull/267) resolves [#257](https://github.com/static-web-server/static-web-server/issues/257) suggested by [@moinologics](https://github.com/moinologics). See [docs](https://static-web-server.net/features/multiple-index-files/).
- [9e50491](https://github.com/static-web-server/static-web-server/commit/9e50491) Maintenance mode support. PR [#272](https://github.com/static-web-server/static-web-server/pull/272) resolves [#268](https://github.com/static-web-server/static-web-server/issues/268) suggested by [@tuxpizza](https://github.com/tuxpizza). See [docs](https://static-web-server.net/features/maintenance-mode/).

__Refactorings__

- [d53c252](https://github.com/static-web-server/static-web-server/commit/d53c252) Optimize buffer size for static file reads (Linux/Unix targets). PR [#269](https://github.com/static-web-server/static-web-server/pull/269).

__Docs__

- [7a407c6](https://github.com/static-web-server/static-web-server/commit/7a407c6) WebAssembly page and Wasmer Wasix example. See [docs](https://static-web-server.net/features/webassembly/).
- [b70058c](https://github.com/static-web-server/static-web-server/commit/b70058c) TrueNAS SCALE installation via TrueCharts. See [docs](https://static-web-server.net/download-and-install/#truenas-scale/).
- [ddbf881](https://github.com/static-web-server/static-web-server/commit/ddbf881) Improve content across several pages.

**Acknowledgments**

Thanks to our new donor [@kirillt](https://github.com/kirillt) for supporting the project.

## v2.22.1 - 2023-09-19

This new `v2.22.1` release brings several dependency updates and bug fixes. In particular, it fixes an issue when capturing glob groups for URL Rewrites and Redirects.

__Fixes__

- [0b5f590](https://github.com/static-web-server/static-web-server/commit/0b5f590) Bugfix/security dependency updates including aho-corasick (regex), clap, syn and other crates.
- [2e3e49f](https://github.com/static-web-server/static-web-server/commit/2e3e49f) URL Rewrites and Redirects do not capture glob groups like `/dir/{*}` correctly. PR [#265](https://github.com/static-web-server/static-web-server/pull/265) resolves [#264](https://github.com/static-web-server/static-web-server/issues/264) reported by [@clembu](https://github.com/clembu).

## v2.22.0 - 2023-09-18

This new `v2.22.0` release brings several dependency updates and bug fixes. It fixes a performance regression leading to better RAM utilization (~28% less) in comparison to the previous releases with a slight req/sec increase, a new Illumos x86_64 target, as well as improved responsiveness of the directory listing HTML page for mobile and desktop screens.

__Fixes__

- [232677c](https://github.com/static-web-server/static-web-server/commit/232677c) Bugfix/security dependency updates including rustls, async-compression, chrono, clap, serde, regex and other crates. Also [b2322a9](https://github.com/static-web-server/static-web-server/commit/b2322a9).

__Features__

- [2ec408c](https://github.com/static-web-server/static-web-server/commit/2ec408c) Illumos x86_64 target. PR [#258](https://github.com/static-web-server/static-web-server/pull/258).

__Refactorings__

- [698a244](https://github.com/static-web-server/static-web-server/commit/698a244) Prefer optional slice references for several `vec` data arguments.
- [257d47f](https://github.com/static-web-server/static-web-server/commit/257d47f) Remove typed headers when appending `cache-control`.
- [48d1910](https://github.com/static-web-server/static-web-server/commit/48d1910) Improve the responsiveness of the directory listing HTML view. PR [#260](https://github.com/static-web-server/static-web-server/pull/260) resolves [#259](https://github.com/static-web-server/static-web-server/issues/259) reported by [@anantakrishna](https://github.com/anantakrishna).
- [e551d67](https://github.com/static-web-server/static-web-server/commit/e551d67) Increase MSRV to 1.70.0.

## v2.21.1 - 2023-08-23

This new `v2.21.1` release brings several security dependency updates. In particular for `serde_derive` and `rustls-webpki` dependencies.

__Fixes__

- [c6172b4](https://github.com/static-web-server/static-web-server/commit/c6172b4) Security dependency updates including serde_derive, rustls-webpki, h2 and other crates.
  - `serde_derive`: potential supply chain attack associated with shipping
precompiled binaries (silently) [serde-rs/serde#2538](https://github.com/serde-rs/serde/issues/2538)
  - `rustls-webpki`: potential CPU denial of service in certificate path building [GHSA-fh2r-99q2-6mmg](https://github.com/advisories/GHSA-fh2r-99q2-6mmg)

## v2.21.0 - 2023-08-19

This new `v2.21.0` release brings several dependency updates and bug fixes, a new NetBSD x86_64 target, Virtual Hosting support, and other improvements.

__Fixes__

- [91d8bf1](https://github.com/static-web-server/static-web-server/commit/91d8bf1) Bugfix/security dependency updates including tokio, regex, clap, serde, globset and other crates.
- [2142053](https://github.com/static-web-server/static-web-server/commit/2142053) Docker: Alpine 3.17.5 update.
- [37a5113](https://github.com/static-web-server/static-web-server/commit/37a5113) Docker: Debian 12.1 update.

__Features__

- [94e050b](https://github.com/static-web-server/static-web-server/commit/94e050b) NetBSD x86_64 target (`x86_64-unknown-netbsd`).
- [7baf569](https://github.com/static-web-server/static-web-server/commit/7baf569) Virtual Hosting support. PR [#252](https://github.com/static-web-server/static-web-server/pull/252) by [@mac-chaffee](https://github.com/mac-chaffee) resolves [#171](https://github.com/static-web-server/static-web-server/issues/171) suggested by [@kshpytsya](https://github.com/kshpytsya). See [docs](https://static-web-server.net/features/virtual-hosting/).

__Docs__

- [3f63a0b](https://github.com/static-web-server/static-web-server/commit/3f63a0b) docs: improve several feature pages.

## v2.20.2 - 2023-08-03

This new `v2.20.2` release brings several dependency updates and bug fixes. Also, it fixes a regression in Windows introduced by the previous *v2.20.1* release.

__Fixes__

- [bba9083](https://github.com/static-web-server/static-web-server/commit/bba9083) Bugfix/security dependency updates including jemallocator, rustls, clap, serde, globset and other crates.
- [8cc073f](https://github.com/static-web-server/static-web-server/commit/8cc073f) Unable to initialize logger in Windows. [#248](https://github.com/static-web-server/static-web-server/issues/248) reported by [@tripplet](https://github.com/tripplet).

__Refactorings__

- [e9d33ca](https://github.com/static-web-server/static-web-server/commit/e9d33ca) Basic-auth check request function.

## v2.20.1 - 2023-07-20

This new `v2.20.1` release brings several dependency updates and bug fixes. In particular, one fix for a regression introduced by the previous *v2.20.0* release as well as other improvements.

__Fixes__

- [1fe464b](https://github.com/static-web-server/static-web-server/commit/1fe464b) Bugfix/security dependency updates including zstd, clap, serde, bcrypt, globset, signal and other crates (also [b763b50](https://github.com/static-web-server/static-web-server/commit/b763b50)).
- [3cf13dc](https://github.com/static-web-server/static-web-server/commit/3cf13dc) URL Rewrites and Redirects don't work properly without replacements. PR [#244](https://github.com/static-web-server/static-web-server/pull/244) fixes [#243](https://github.com/static-web-server/static-web-server/issues/243) reported by [@domi2120](https://github.com/domi2120).
- [8da2b69](https://github.com/static-web-server/static-web-server/commit/8da2b69) Alpine 3.17.4.

__Refactorings__

- [949c539](https://github.com/static-web-server/static-web-server/commit/949c539) Initialize log system at config level.
- [7fc0e1b](https://github.com/static-web-server/static-web-server/commit/7fc0e1b) Improve start-up server log information.
- [032aaf3](https://github.com/static-web-server/static-web-server/commit/032aaf3) CI: Post-release script and devel Makefile.

## v2.20.0 - 2023-07-12

This new `v2.20.0` release brings several dependency updates and bug fixes, advanced features like Glob pattern replacements for URL Redirects and Rewrites, a new health-check endpoint, GitHub Container Registry (GHCR) Docker images as well as other improvements.

__Fixes__

- [9b84786](https://github.com/static-web-server/static-web-server/commit/9b84786) Bugfix/security dependency updates including tokio, hyper, h2, rustls, clap, serde, toml and other crates (also [9b84786](https://github.com/static-web-server/static-web-server/commit/9b84786)).
- [b8473aa](https://github.com/static-web-server/static-web-server/commit/b8473aa) Potential panic when invalid content range.
- [2331c88](https://github.com/static-web-server/static-web-server/commit/2331c88) CI: Post-release update script.

__Features__

- [4a10635](https://github.com/static-web-server/static-web-server/commit/4a10635) Docker: GitHub Container Registry (GHCR) Docker images. PR [#232](https://github.com/static-web-server/static-web-server/pull/232) resolves [#225](https://github.com/static-web-server/static-web-server/issues/225) suggested by [@jcgruenhage](https://github.com/jcgruenhage). See [docs](https://static-web-server.net/features/docker/).
- [06955e9](https://github.com/static-web-server/static-web-server/commit/06955e9) Redirect option for URL Rewrites feature. PR [#231](https://github.com/static-web-server/static-web-server/pull/231). See [docs](https://static-web-server.net/features/health-endpoint/).
- [3a47ef6](https://github.com/static-web-server/static-web-server/commit/3a47ef6) Replacements support for URL Rewrites destination. PR [#235](https://github.com/static-web-server/static-web-server/pull/235). See [docs](https://static-web-server.net/features/url-rewrites/).
- [7c66c5c](https://github.com/static-web-server/static-web-server/commit/7c66c5c) Replacements support for URL Redirects destination. PR [#239](https://github.com/static-web-server/static-web-server/pull/239). See [docs](https://static-web-server.net/features/url-redirects/).
- [b42214b](https://github.com/static-web-server/static-web-server/commit/b42214b) Health-check endpoint. PR [#238](https://github.com/static-web-server/static-web-server/pull/238) resolves [#237](https://github.com/static-web-server/static-web-server/issues/237) by [@glehmann](https://github.com/glehmann). See [docs](https://static-web-server.net/features/health-endpoint/).

__Refactorings__

- [1bce204](https://github.com/static-web-server/static-web-server/commit/1bce204) Improve auto index options.
- [b2e4e49](https://github.com/static-web-server/static-web-server/commit/b2e4e49) Improve directory listing styling for HTML display.
- [e23a06d](https://github.com/static-web-server/static-web-server/commit/e23a06d) Lib: Crate docs metadata.

__Docs__

- [506f54e](https://github.com/static-web-server/static-web-server/commit/506f54e) Systemd service example. See [docs](https://static-web-server.net/features/file-descriptor-socket-passing/#service-example).
- [eb2887f](https://github.com/static-web-server/static-web-server/commit/eb2887f) Nix package and module maintainers ([@figsoda](https://github.com/figsoda), [@mac-chaffee](https://github.com/mac-chaffee)). See [docs](https://static-web-server.net/download-and-install/#nixos).
- [031931f](https://github.com/static-web-server/static-web-server/commit/031931f) GHCR Docker images description. See [docs](https://static-web-server.net/features/docker).
- [21c90db](https://github.com/static-web-server/static-web-server/commit/21c90db) Several documentation improvements.

**Acknowledgments**

Thanks to our new donor [@kirillt](https://github.com/kirillt) for supporting the project.

## v2.19.0 - 2023-06-16

This new `v2.19.0` release brings several dependency updates/bug fixes (including minor versions), a new [Debian 12 "bookworm"](https://www.debian.org/News/2023/20230610) Docker image, more Cargo features for controlling the SWS feature set when building, bug fixes for the SWS crate and one regression for the `fallback-page` feature, documentation for cross-compiling SWS from source using [Zig as a linker](https://andrewkelley.me/post/zig-cc-powerful-drop-in-replacement-gcc-clang.html) as well as other improvements.

__Fixes__

- [d258803](https://github.com/static-web-server/static-web-server/commit/d258803) Bugfix/security dependency updates including clap, async-compression, zstd, tokio-rustls, toml, pin-project, form_urlencoded, percent-encoding and other crates.
- [3e4bd47](https://github.com/static-web-server/static-web-server/commit/3e4bd47) Value is required for `fallback-page` when using empty values. PR [#219](https://github.com/static-web-server/static-web-server/pull/219) fixes [#218](https://github.com/static-web-server/static-web-server/issues/218) reported by [@OdyX](https://github.com/OdyX)
- [558fd96](https://github.com/static-web-server/static-web-server/commit/558fd96) Lib: Unresolved/unused imports when disabling all Cargo features.
- [911a1c2](https://github.com/static-web-server/static-web-server/commit/911a1c2) Misc: Fix some module typos.
- [b751b40](https://github.com/static-web-server/static-web-server/commit/b751b40) CI: Wrong release tag for checksums workflow.

__Features__

- [3adf75e](https://github.com/static-web-server/static-web-server/commit/3adf75e) Docker: [Debian 12 "bookworm"](https://www.debian.org/News/2023/20230610) Docker image using statically-linked binary (musl libc). See [docs](https://static-web-server.net/features/docker/).
- [79a93f6](https://github.com/static-web-server/static-web-server/commit/79a93f6) Lib: `directory-listing` Cargo feature. PR [#220](https://github.com/static-web-server/static-web-server/pull/220). See [docs](https://static-web-server.net/building-from-source/#building-project-from-source).
- [a8144d6](https://github.com/static-web-server/static-web-server/commit/a8144d6) Lib: `basic-auth` Cargo feature. PR [#221](https://github.com/static-web-server/static-web-server/pull/221). See [docs](https://static-web-server.net/building-from-source/#building-project-from-source).
- [680c8aa](https://github.com/static-web-server/static-web-server/commit/680c8aa) Lib: `fallback-page` Cargo feature. PR [#222](https://github.com/static-web-server/static-web-server/pull/222). See [docs](https://static-web-server.net/building-from-source/#building-project-from-source).

__Refactorings__

- [986b663](https://github.com/static-web-server/static-web-server/commit/986b663) Lib: Enable Crate `docsrs` config flag.
- [9e635bd](https://github.com/static-web-server/static-web-server/commit/9e635bd) Lib: Improve Cargo docs for some SWS features.
- [a0f92f5](https://github.com/static-web-server/static-web-server/commit/a0f92f5) CI: Post-release updates script.

__Docs__

- [379f88b](https://github.com/static-web-server/static-web-server/commit/379f88b) Cross-compile SWS from source using [Zig as a linker](https://andrewkelley.me/post/zig-cc-powerful-drop-in-replacement-gcc-clang.html). See [docs](https://static-web-server.net/building-from-source/#cross-compiling).

## v2.18.0 - 2023-06-07

This new `v2.18.0` release brings several dependency updates/bug fixes, bug fixes for the `security-headers` and `page-fallback` features, the C runtime in Windows x86_64 is now statically linked, possibility to use CLI boolean flags without explicit values as well as some refactorings and improvements.

__Fixes__

- [ddfc00b](https://github.com/static-web-server/static-web-server/commit/ddfc00b) Bugfix/security dependency updates including clap, parking_lot, libc, percent-encoding, form_urlencoded, regex and other crates.
- [cbb21c0](https://github.com/static-web-server/static-web-server/commit/cbb21c0) `security-headers` not enabled by default when using `http2` via config file. PR [#216](https://github.com/static-web-server/static-web-server/pull/216) fixes [#210](https://github.com/static-web-server/static-web-server/issues/210) resported by [@mac-chaffee](https://github.com/mac-chaffee).
- [91519c9](https://github.com/static-web-server/static-web-server/commit/91519c9) Obsolete `X-XSS-Protection` header on `security-headers` (also [d5279ff](https://github.com/static-web-server/static-web-server/commit/d5279ff)). Reported on [#213](https://github.com/static-web-server/static-web-server/discussions/213) by [@picchietti](https://github.com/picchietti).
- [e183ea3](https://github.com/static-web-server/static-web-server/commit/e183ea3) Missing SWS base modules when `page-fallback` is enabled. Reported on [#213](https://github.com/static-web-server/static-web-server/discussions/213) by [@picchietti](https://github.com/picchietti).<br>
  The following SWS modules are now used when `page-fallback` feature is activated:
  - `cors`
  - `compression`
  - `cache_control_headers`
  - `security_headers`
  - `custom_headers`
- [fba6665](https://github.com/static-web-server/static-web-server/commit/fba6665) CI: Workflow fails to generate proper checksums.

__Features__

- [2150c74](https://github.com/static-web-server/static-web-server/commit/2150c74) Support for CLI boolean flags without explicit values (E.g. `static-web-server -d public/ --compression -z`). PR [#215](https://github.com/static-web-server/static-web-server/pull/215) resolves [#209](https://github.com/static-web-server/static-web-server/issues/209) suggested by [@mac-chaffee](https://github.com/mac-chaffee).

__Refactorings__

- [fa0cca5](https://github.com/static-web-server/static-web-server/commit/fa0cca5) Statically link the C runtime on Windows MSVC x86_64 to avoid the `VCRUNTIME140.dll`.
- [a75147e](https://github.com/static-web-server/static-web-server/commit/a75147e) Lib: Rust nightly toolchain for crate docs.
- [520e66d](https://github.com/static-web-server/static-web-server/commit/520e66d) CI: Increase verbosity of `cargo build` across pipelines.

__Acknowledgments__

Thanks to our new donor [@picchietti](https://github.com/picchietti) for supporting the project.

## v2.17.0 - 2023-06-03

This new `v2.17.0` release brings several dependency updates/bug fixes, ECC private keys support for the `tls` feature, HTTP to HTTPS redirect support, several Cargo features for controlling the SWS `compression` and `compression-static`, dependency migrations like the `clap` CLI parser and `tokio-rustls` as well as various refactorings and improvements.

__Fixes__

- [b685cda](https://github.com/static-web-server/static-web-server/commit/b685cda) Bugfix/security dependency updates including tokio, tracing, chrono, serde, h2, libc, pin-project, windows-sys and other crates.

__Features__

- [946b4e5](https://github.com/static-web-server/static-web-server/commit/946b4e5) HTTP to HTTPS redirect support. PR [#203](https://github.com/static-web-server/static-web-server/pull/203) resolves [#202](https://github.com/static-web-server/static-web-server/issues/202) suggested by [@micsama](https://github.com/micsama). See [docs](https://static-web-server.net/features/http-https-redirect/).
- [0f66443](https://github.com/static-web-server/static-web-server/commit/0f66443) ECC private keys support for the `tls` feature. PR [#208](https://github.com/static-web-server/static-web-server/pull/208) resolves [#207](https://github.com/static-web-server/static-web-server/issues/207) suggested by [@mac-chaffee](https://github.com/mac-chaffee). See [docs](https://static-web-server.net/features/http2-tls/#private-key-file-formats).
- [af77e4a](https://github.com/static-web-server/static-web-server/commit/af77e4a) Lib: Cargo features for `compression` and `compression-static`. PR [#201](https://github.com/static-web-server/static-web-server/pull/201).
- [f8fca0a](https://github.com/static-web-server/static-web-server/commit/f8fca0a) Misc: Include SPDX license identifiers in every source file.
- [a345df3](https://github.com/static-web-server/static-web-server/commit/a345df3) Misc: Benchmarks 2023-04. See [repository](https://github.com/static-web-server/benchmarks).
- [1894474](https://github.com/static-web-server/static-web-server/commit/1894474) CI: Workflow to automate checksums.

__Refactorings__

- [4e01de6](https://github.com/static-web-server/static-web-server/commit/4e01de6) Migrate `clap` CLI parser to v3. PR [#211](https://github.com/static-web-server/static-web-server/pull/211) by [@mac-chaffee](https://github.com/mac-chaffee).
- [e8560a0](https://github.com/static-web-server/static-web-server/commit/e8560a0) Update `tokio-rustls` to `0.24`.
- [20de5d0](https://github.com/static-web-server/static-web-server/commit/20de5d0) HTTP to HTTPS redirect feature improvements.
- [647e9b0](https://github.com/static-web-server/static-web-server/commit/647e9b0) Lib: Include missing `rustls-pemfile` in Cargo `tls` feature.
- [53ef76e](https://github.com/static-web-server/static-web-server/commit/53ef76e) Lib: Improve Rust docs for Cargo features.
- [6b81c48](https://github.com/static-web-server/static-web-server/commit/6b81c48) Lib: Simplify `http2` Cargo feature.
- [ae17023](https://github.com/static-web-server/static-web-server/commit/ae17023) CI: Simplify workflow scripts.

__Docs__

- [d3fa602](https://github.com/static-web-server/static-web-server/commit/d3fa602) HTTP to HTTPS redirect feature page. See [docs](https://static-web-server.net/features/http-https-redirect/).
- [880eaf4](https://github.com/static-web-server/static-web-server/commit/880eaf4) HTTP/2 and TLS feature page improvements. See [docs](https://static-web-server.net/features/http-https-redirect/). See [docs](https://static-web-server.net/features/http2-tls/).
- [e0ae5a7](https://github.com/static-web-server/static-web-server/commit/e0ae5a7) Blocking threads feature page. See [docs](https://static-web-server.net/features/blocking-threads/).
- [c64e3d6](https://github.com/static-web-server/static-web-server/commit/c64e3d6) Safe TLS defaults description. See [docs](https://static-web-server.net/features/http2-tls/#safe-tls-defaults).
- [6876a75](https://github.com/static-web-server/static-web-server/commit/6876a75) Enable content editing option and revision. See [docs](https://static-web-server.net/).

## v2.16.0 - 2023-04-25

This new `v2.16.0` release brings several dependency updates/bug fixes including the Alpine Docker image, a new Android ARM64 target, Zstandard (zstd) auto-compression and pre-compressed files support, `static-web-server` available as a crate, as well as other additions and improvements.

__Fixes__

- [44daf6b](https://github.com/static-web-server/static-web-server/commit/44daf6b) Bugfix/security dependency updates including hyper, tokio, futures, serde, h2, libc, windows and other crates. Also ([be8ba9b](https://github.com/static-web-server/static-web-server/commit/be8ba9b), [fff3d4e](https://github.com/static-web-server/static-web-server/commit/fff3d4e)).
- [39cfbab](https://github.com/static-web-server/static-web-server/commit/39cfbab) Improve error handling when reading file entries on the directory listing module. PR [#192](https://github.com/static-web-server/static-web-server/pull/192) fixes [#191](https://github.com/static-web-server/static-web-server/issues/191) reported by [@PlkMarudny](https://github.com/PlkMarudny).
- [e36a522](https://github.com/static-web-server/static-web-server/commit/e36a522) Docker: Update Alpine to 3.17.3. PR [#199](https://github.com/static-web-server/static-web-server/pull/199) by [@gaby](https://github.com/gaby).
- [9d7de82](https://github.com/static-web-server/static-web-server/commit/9d7de82) CI: Error when installing the latest Rust on FreeBSD.

__Features__

- [70db3c9](https://github.com/static-web-server/static-web-server/commit/70db3c9) New [static-web-server](https://crates.io/crates/static-web-server) crate. PR [#190](https://github.com/static-web-server/static-web-server/pull/190) resolves [#188](https://github.com/static-web-server/static-web-server/issues/188) suggested by [@da2ce7](https://github.com/da2ce7).
- [d7dd255](https://github.com/static-web-server/static-web-server/commit/d7dd255) New Android ARM64 target support (`aarch64-linux-android`). PR [#194](https://github.com/static-web-server/static-web-server/pull/194) resolves [#163](https://github.com/static-web-server/static-web-server/issues/163) suggested by [@denisidoro](https://github.com/denisidoro).
- [2bebec7](https://github.com/static-web-server/static-web-server/commit/2bebec7) Zstandard (`zstd`) auto-compression and pre-compressed files support. PR [#197](https://github.com/static-web-server/static-web-server/pull/197) resolves [#193](https://github.com/static-web-server/static-web-server/issues/193) suggested by [@gaby](https://github.com/gaby).
- [910eaae](https://github.com/static-web-server/static-web-server/commit/910eaae) Add `Vary` header for `Accept-Encoding` when `--compression` or `--compression-static` is enabled.
- [ca5e7f5](https://github.com/static-web-server/static-web-server/commit/ca5e7f5) CI: Workflow for publishing the `static-web-server` crate. PR [#189](https://github.com/static-web-server/static-web-server/pull/189) by [@da2ce7](https://github.com/da2ce7).
- [415465c](https://github.com/static-web-server/static-web-server/commit/415465c) Misc: Binary installer script for Linux/BSDs. See [docs](https://static-web-server.net/download-and-install/#binary-installer-linuxbsds).

__Refactorings__

- [e751bfb](https://github.com/static-web-server/static-web-server/commit/e751bfb) Remove needless `as_ref()` for several paths on static file module.
- [cc1de08](https://github.com/static-web-server/static-web-server/commit/cc1de08) Rename `.html` auto-suffix files metadata function.
- [981c388](https://github.com/static-web-server/static-web-server/commit/981c388) CI: Runner images and dependency updates.
- [4a12898](https://github.com/static-web-server/static-web-server/commit/4a12898) CI: Improve devel/prod pipelines.
- [cf0d618](https://github.com/static-web-server/static-web-server/commit/cf0d618) Misc: Logo and project description.
- [90ec4b6](https://github.com/static-web-server/static-web-server/commit/90ec4b6) Misc: Move the website to [static-web-server.net](https://static-web-server.net/).

__Docs__

- [337f652](https://github.com/static-web-server/static-web-server/commit/337f652) Describe `v1.x` end of life (2023-01-06). See [static-web-server.net](https://static-web-server.net/).
- [85851e9](https://github.com/static-web-server/static-web-server/commit/85851e9) Zstd compression feature description.
- [29b3587](https://github.com/static-web-server/static-web-server/commit/29b3587) Minor README grammatical & readability updates. PR [#196](https://github.com/static-web-server/static-web-server/pull/196) by [@dabrd](https://github.com/dabrd).
- [32398b4](https://github.com/static-web-server/static-web-server/commit/32398b4) Update README page links.

## v2.15.0 - 2023-03-13

This new `v2.15.0` release brings several dependency updates, one bug fix for the `compression-static`, new features like  Tokio's `--max-blocking-threads` or `.html` prefixing for directory requests, the possibility to build SWS on non-Unix/Windows platforms and performance optimizations and improvements across several modules including `static_file` which [speeds up](https://gist.github.com/joseluisq/cb0962474210e56e768ff5671b3ddd11) SWS around `~4.37%` (req/sec) for almost the same computing.

__Fixes__

- [5d49c09](https://github.com/static-web-server/static-web-server/commit/5d49c09) Bugfix/security dependency updates including hyper, tokio, futures, serde, h2, listenfd, windows-service, bcrypt, chrono and other crates.
- [06cba46](https://github.com/static-web-server/static-web-server/commit/06cba46) Compression static auto `index.html` check missing. PR [#186](https://github.com/static-web-server/static-web-server/pull/186) resolves [#178](https://github.com/static-web-server/static-web-server/issues/178) reported by [@glehmann](https://github.com/glehmann).

__Features__

- [40a532e](https://github.com/static-web-server/static-web-server/commit/40a532e) Nix installation support. See [docs](https://static-web-server.net/download-and-install/#nixos).
- [b9fa2bf](https://github.com/static-web-server/static-web-server/commit/b9fa2bf) Support for Tokio's `--max-blocking-threads` via new option. PR [#181](https://github.com/static-web-server/static-web-server/pull/181) by [@syrusakbary](https://github.com/syrusakbary), [@john-sharratt](https://github.com/john-sharratt).
- [7ed7b03](https://github.com/static-web-server/static-web-server/commit/7ed7b03) Support for `.html` prefixing when a request path doesn't exist. PR [#180](https://github.com/static-web-server/static-web-server/pull/180) by [@syrusakbary](https://github.com/syrusakbary).
- [87a0896](https://github.com/static-web-server/static-web-server/commit/87a0896) Optional `http2` Cargo feature. PR [#183](https://github.com/static-web-server/static-web-server/pull/183) by [@syrusakbary](https://github.com/syrusakbary), [@john-sharratt](https://github.com/john-sharratt).

__Refactorings__

- [b2cff1b](https://github.com/static-web-server/static-web-server/commit/b2cff1b) Optimize cache control headers file type detection. PR [#175](https://github.com/static-web-server/static-web-server/pull/175) by [@mfontanini](https://github.com/mfontanini).
- [9796d35](https://github.com/static-web-server/static-web-server/commit/9796d35) Several performance optimizations and code improvements. PR [#177](https://github.com/static-web-server/static-web-server/pull/177) by [@mfontanini](https://github.com/mfontanini).
- [22123c1](https://github.com/static-web-server/static-web-server/commit/22123c1) Make signals support optional for non-Unix/Windows targets. PR [#185](https://github.com/static-web-server/static-web-server/pull/185) by [@syrusakbary](https://github.com/syrusakbary), [@john-sharratt](https://github.com/john-sharratt).
- [7490697](https://github.com/static-web-server/static-web-server/commit/7490697) Improve `compression_static` module's result type.
- [7c68b8c](https://github.com/static-web-server/static-web-server/commit/7c68b8c) Improve `static_file` module's composed file metadata.
- [200fce0](https://github.com/static-web-server/static-web-server/commit/200fce0) Enable new Cargo's `sparse` protocol on CI for devel/prod pipelines.
- [db063e4](https://github.com/static-web-server/static-web-server/commit/db063e4) Replace unmaintained `actions-rs/clippy-check` on CI.

__Docs__

- [a4250fd](https://github.com/static-web-server/static-web-server/commit/a4250fd) Mention Cargo features when running or building from source.

__Acknowledgments__

Thanks to our new donnors [@marcusbuffett](https://github.com/marcusbuffett) and [@scottsweb](https://github.com/scottsweb) for support the project.

## v2.14.2 - 2023-02-15

__Fixes__

- [495f3ae](https://github.com/static-web-server/static-web-server/commit/495f3ae) Bugfix/security dependency updates including hyper, tokio, tikv-jemallocator, futures, rustls, toml, serde, parking_lot and other crates. Also ([41a9b0b](https://github.com/static-web-server/static-web-server/commit/41a9b0b)).
- [7b81f0c](https://github.com/static-web-server/static-web-server/commit/7b81f0c) Explicit Alpine 3.16.4 Docker images update.
- [e81b277](https://github.com/static-web-server/static-web-server/commit/e81b277) Unnecessary u32 cast on directory listing last modified function.

__Refactorings__

- [2cf9008](https://github.com/static-web-server/static-web-server/commit/2cf9008) Remove TOML incompatibility hacks.
- [d9f9204](https://github.com/static-web-server/static-web-server/commit/d9f9204) Minor clippy syntax and format improvements.
- [190db74](https://github.com/static-web-server/static-web-server/commit/190db74) Project Minimum Supported Rust Version (MSRV) is now 1.66.0.

## v2.14.1 - 2022-12-02

__Fixes__

- [e723716](https://github.com/static-web-server/static-web-server/commit/e723716) Bugfix/security dependency updates including tokio-macros, flate2, parking_lot, serde and other crates.
- [b431c68](https://github.com/static-web-server/static-web-server/commit/b431c68) Panic on compression-static when root dir is a dot and client supports compression. PR [#166](https://github.com/static-web-server/static-web-server/pull/166) fixes [#165](https://github.com/static-web-server/static-web-server/issues/165) by [@eduardo-gomes](https://github.com/eduardo-gomes).

## v2.14.0 - 2022-11-22

__Fixes__

- [ff69788](https://github.com/static-web-server/static-web-server/commit/ff69788) Bugfix/security dependency updates including tokio, hyper, chrono, listenfd, num_cpus, windows-service, serde, regex and other crates. (also [d1baad6](https://github.com/static-web-server/static-web-server/commit/d1baad6), [efda237](https://github.com/static-web-server/static-web-server/commit/efda237), [676d7e9](https://github.com/static-web-server/static-web-server/commit/676d7e9))
- [9d67d9d](https://github.com/static-web-server/static-web-server/commit/9d67d9d) Unhandled panic when get "last modified" info on `directory_listing` module.
- [6f059fd](https://github.com/static-web-server/static-web-server/commit/6f059fd) Needless borrow on `signals` module.
- [edc3fdf](https://github.com/static-web-server/static-web-server/commit/edc3fdf) Update CI `cross` dependency to latest 0.2.x.

__Breaking__

- [a09ff1f](https://github.com/static-web-server/static-web-server/commit/a09ff1f) Add missing `SERVER` prefix for the `REDIRECT_TRAILING_SLASH` env. PR [#161](https://github.com/static-web-server/static-web-server/pull/161).<br>
  This is a breaking change *only* if the previous `REDIRECT_TRAILING_SLASH` env was used explicitly.<br>
  Otherwise, if not set/used (default behavior) or using the equivalent CLI argument then there is no impact or action required.<br>
  However, we highly encourage users to prefer `SERVER_REDIRECT_TRAILING_SLASH` env instead.

__Features__

- [800416d](https://github.com/static-web-server/static-web-server/commit/800416d) Ignore hidden files/directories (dotfiles) via the new `--ignore-hidden-files` option. PR [#162](https://github.com/static-web-server/static-web-server/pull/162). See [docs](https://static-web-server.net/features/ignore-files/).

__Refactorings__

- [6798ff7](https://github.com/static-web-server/static-web-server/commit/6798ff7) Reduce allocations when using the fixed HTTP method list.
- [2828f58](https://github.com/static-web-server/static-web-server/commit/2828f58) Strip symbols on release profile via Cargo.
- [ea4c24c](https://github.com/static-web-server/static-web-server/commit/ea4c24c) Reorder imports on TLS module.
- [0e538dd](https://github.com/static-web-server/static-web-server/commit/0e538dd) Introduce http-related extension traits. PR [#160](https://github.com/static-web-server/static-web-server/pull/160).
- [fb3fb23](https://github.com/static-web-server/static-web-server/commit/fb3fb23) Move project to its [static-web-server](https://github.com/static-web-server) organization.
- [5435f3c](https://github.com/static-web-server/static-web-server/commit/5435f3c) Simplify FreeBSD test and release CI pipelines.
- [d66494c](https://github.com/static-web-server/static-web-server/commit/d66494c) Project files clean up.

__Docs__

- [2fc36b4](https://github.com/static-web-server/static-web-server/commit/2fc36b4) Benchmarks page. PR [#155](https://github.com/static-web-server/static-web-server/pull/155).
- [5097738](https://github.com/static-web-server/static-web-server/commit/5097738) Clarify benchmark context and remarks. PR [#157](https://github.com/static-web-server/static-web-server/pull/157) resolves [#156](https://github.com/static-web-server/static-web-server/issues/156) suggested by [@mufeedvh](https://github.com/mufeedvh).
- [70f37f6](https://github.com/static-web-server/static-web-server/commit/70f37f6) Minor environment variables fixes. PR [#158](https://github.com/static-web-server/static-web-server/pull/158) by [@funkyfuture](https://github.com/funkyfuture).

**Advice about the new organization change**

Certainly, there is no impact if you still rely on previous GitHub release links (E.g pre-compiled binaries) because they are always redirected permanently.<br>
However, since we moved to a [new organization](https://github.com/static-web-server/static-web-server), we highly encourage you to update your links using the new GitHub release address of the `static-web-server` organization.

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

- [f369c80](https://github.com/static-web-server/static-web-server/commit/f369c80) CORS exposed headers support via new `--cors-expose-headers` option. PR [#144](https://github.com/static-web-server/static-web-server/pull/144) by [@nelsonjchen](https://github.com/nelsonjchen). See [docs](https://static-web-server.net/features/cors/#exposed-headers).
- [997e493](https://github.com/static-web-server/static-web-server/commit/997e493) HTML/JSON support for directory listing entries via new `--directory-listing-format` option. PR [#151](https://github.com/static-web-server/static-web-server/pull/151). See [docs](https://static-web-server.net/features/directory-listing/#output-format).

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

- [91b6ba2](https://github.com/static-web-server/static-web-server/commit/91b6ba2) Relative paths for directory listing entries. PR [#137](https://github.com/static-web-server/static-web-server/pull/137) resolves [#136](https://github.com/static-web-server/static-web-server/issues/136) suggested by [@jtackaberry](https://github.com/jtackaberry). See [docs](https://static-web-server.net/features/directory-listing/#relative-paths-for-entries).
- [5f10771](https://github.com/static-web-server/static-web-server/commit/5f10771) Log Real Remote IP in case of proxies. PR [#138](https://github.com/static-web-server/static-web-server/pull/138) by [@dlvoy](https://github.com/dlvoy). See [docs](https://static-web-server.net/features/logging/#log-real-remote-ip).
- [48f9458](https://github.com/static-web-server/static-web-server/commit/48f9458) Support for serving pre-compressed (Gzip/Brotli) files. PR [#139](https://github.com/static-web-server/static-web-server/pull/139) resolves [#114](https://github.com/static-web-server/static-web-server/issues/114) suggested by [@JonasGilg](https://github.com/JonasGilg). See [docs](https://static-web-server.net/features/compression-static/).

__Refactorings__

- [e9a4aa3](https://github.com/static-web-server/static-web-server/commit/e9a4aa3) Directory listing module.
- [eee45f9](https://github.com/static-web-server/static-web-server/commit/eee45f9) Remove indirections on static file module (performance improvement).

__Docs__

- [59a75e6](https://github.com/static-web-server/static-web-server/commit/59a75e6) Fix documentation typos. PR [#140](https://github.com/static-web-server/static-web-server/pull/140) by [@kianmeng](https://github.com/kianmeng).
- [3ca743a](https://github.com/static-web-server/static-web-server/commit/3ca743a) Page for pre-compressed files serving feature. See [docs](https://static-web-server.net/features/compression-static/).
- [88a886f](https://github.com/static-web-server/static-web-server/commit/88a886f) Building project from source now requires Rust `1.59.0` or later. See [docs](https://static-web-server.net/building-from-source/#building-project-from-source).

__Codebase__
 
- [5148da9](https://github.com/static-web-server/static-web-server/commit/5148da9) CI workflow for Rust security audit.
- [28f8818](https://github.com/static-web-server/static-web-server/commit/28f8818) CI development job for typos. PR [#141](https://github.com/static-web-server/static-web-server/pull/141) by [@kianmeng](https://github.com/kianmeng). See [docs](https://static-web-server.net/features/logging/#log-real-remote-ip).

## v2.11.0 - 2022-08-15

__Fixes__

- [1b7636c](https://github.com/static-web-server/static-web-server/commit/1b7636c) Bugfix/security dependency updates including tokio, serde, tracing, libc, futures and other crates (also [6840d0f](https://github.com/static-web-server/static-web-server/commit/6840d0f), [32517b6](https://github.com/static-web-server/static-web-server/commit/32517b6)).
- [6570498](https://github.com/static-web-server/static-web-server/commit/6570498) Enable the missing `windows-service` option when used via the config file.

__Features__

- [5163564](https://github.com/static-web-server/static-web-server/commit/5163564) New `redirect-trailing-slash` option. PR [#131](https://github.com/static-web-server/static-web-server/pull/131) by [@phartenfeller](https://github.com/phartenfeller). See [docs](https://static-web-server.net/features/trailing-slash-redirect/).

__Docs__

- [10f403f](https://github.com/static-web-server/static-web-server/commit/10f403f) Redirect trailing slash page.
- [e4228af](https://github.com/static-web-server/static-web-server/commit/e4228af) Typos and content improvements (also [e674940](https://github.com/static-web-server/static-web-server/commit/e674940)).

## v2.10.0 - 2022-07-10

__Fixes__

- [b902cb7](https://github.com/static-web-server/static-web-server/commit/b902cb7) Bugfix/security dependency updates including tokio, hyper, tracing, jemallocator and other crates (also [5c9b797](https://github.com/static-web-server/static-web-server/commit/5c9b797), [4cf9a6b](https://github.com/static-web-server/static-web-server/commit/4cf9a6b)).
- [b73959f](https://github.com/static-web-server/static-web-server/commit/b73959f) Fix wrong prefix config file path (`\\?\`) when logging on Windows.

__Features__

- [5163564](https://github.com/static-web-server/static-web-server/commit/5163564) URL Rewrites with pattern matching support. PR [#122](https://github.com/static-web-server/static-web-server/pull/122). See [docs](https://static-web-server.net/features/url-rewrites/).
- [5ef3b62](https://github.com/static-web-server/static-web-server/commit/5ef3b62) URL Redirects with pattern matching. PR [#123](https://github.com/static-web-server/static-web-server/pull/123). See [docs](https://static-web-server.net/features/url-rewrites/).
- [9072977](https://github.com/static-web-server/static-web-server/commit/9072977) Homebrew installation support for MacOS/Linux. See [docs](https://static-web-server.net/download-and-install/#macos).
- [975132f](https://github.com/static-web-server/static-web-server/commit/975132f) [Scoop](https://scoop.sh/#/apps?q=static-web-server&s=0&d=1&o=true) installation support for Windows. See [docs](https://static-web-server.net/download-and-install/#windows).
- [78a5611](https://github.com/static-web-server/static-web-server/commit/78a5611) Alpine 3.16 Docker image.

__Docs__

- [b0ca3d1](https://github.com/static-web-server/static-web-server/commit/b0ca3d1) Several doc typo fixes.

## v2.9.0 - 2022-05-28

__Fixes__

- [446576a](https://github.com/static-web-server/static-web-server/commit/446576a) Bugfix/security dependency updates including tokio, hyper, rustls, compression, windows-rs, serde, log and other crates (also [fa531a0](https://github.com/static-web-server/static-web-server/commit/fa531a0), [0879c84](https://github.com/static-web-server/static-web-server/commit/0879c84)).

__Features__

- [3d1776d](https://github.com/static-web-server/static-web-server/commit/3d1776d) Windows Service support via new `--windows-service` option. PR [#110](https://github.com/static-web-server/static-web-server/pull/110) resolves [#65](https://github.com/static-web-server/static-web-server/issues/65) suggested by [@bubnenkoff](https://github.com/bubnenkoff). See [docs](https://static-web-server.net/features/windows-service/).
- [bd78034](https://github.com/static-web-server/static-web-server/commit/bd78034) Include request URI on tracing log for 404/50x errors. [#108](https://github.com/static-web-server/static-web-server/issues/108) suggested by [@stappersg](https://github.com/stappersg).
- [b49395a](https://github.com/static-web-server/static-web-server/commit/b49395a) Log request file with its remote address (IP) via new `--log-remote-address` option. PR [#112](https://github.com/static-web-server/static-web-server/pull/112) resolves [#111](https://github.com/static-web-server/static-web-server/issues/111) suggested by [@nicheath](https://github.com/nicheath). See [docs](https://static-web-server.net/features/logging/#log-remote-addresses).

__Docs__

- [a793b58](https://github.com/static-web-server/static-web-server/commit/a793b58) Improve basic auth feature page. See [docs](https://static-web-server.net/features/basic-authentication/).
- [ae0dcfd](https://github.com/static-web-server/static-web-server/commit/ae0dcfd) Windows Service feature page. See [docs](https://static-web-server.net/features/windows-service/).
- [2d71de6](https://github.com/static-web-server/static-web-server/commit/2d71de6) Log remote address feature. See [docs](https://static-web-server.net/features/logging/#log-remote-addresses).

## v2.8.0 - 2022-05-04

__Fixes__

- [446576a](https://github.com/static-web-server/static-web-server/commit/446576a) Bugfix/security dependency updates including http, tokio, httparse, windows-rs, serde, log and other crates.

__Features__

- [1fd3e48](https://github.com/static-web-server/static-web-server/commit/1fd3e48) Configuration file support. PR [#101](https://github.com/static-web-server/static-web-server/pull/101). See [docs](https://static-web-server.net/configuration/config-file/).
- [62ebe52](https://github.com/static-web-server/static-web-server/commit/62ebe52) Custom HTTP headers via config file. See [docs](https://static-web-server.net/features/custom-http-headers/).

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

- [da85b16](https://github.com/static-web-server/static-web-server/commit/da85b16) CORS allowed headers support via the new `-j, --cors-allow-headers` flags. PR [#87](https://github.com/static-web-server/static-web-server/pull/87). See [docs](https://static-web-server.net/features/cors/#allowed-headers).
- [da85b16](https://github.com/static-web-server/static-web-server/commit/da85b16) Support for HTTP `OPTIONS` method requests. PR [#87](https://github.com/static-web-server/static-web-server/pull/87). See [docs](https://static-web-server.net/features/http-methods/).
- [6204205](https://github.com/static-web-server/static-web-server/commit/6204205) `Cache-Control` for AVIF and JPEG XL mime types. PR [#88](https://github.com/static-web-server/static-web-server/pull/88) by [@csmith](https://github.com/csmith). See [docs](https://static-web-server.net/features/cache-control-headers/#one-year).
- [cba4a83](https://github.com/static-web-server/static-web-server/commit/cba4a83) Fallback page option via the new `--page-fallback` flag. PR [#91](https://github.com/static-web-server/static-web-server/pull/91) by [@firstdorsal](https://github.com/firstdorsal). See [docs](https://static-web-server.net/features/error-pages/#fallback-page-for-use-with-client-routers).

__Refactorings__

- [d33d093](https://github.com/static-web-server/static-web-server/commit/d33d093) Reduce few allocations on HTTP request handler.
- [06cc379](https://github.com/static-web-server/static-web-server/commit/06cc379) Reduce small allocation when encoding headers during compression.
- [a5e87e5](https://github.com/static-web-server/static-web-server/commit/a5e87e5) Typed `Content-Type` header for error pages and dir listing responses.

__Docs__

- [781ba91](https://github.com/static-web-server/static-web-server/commit/781ba91) CORS allowed headers. See [docs](https://static-web-server.net/features/cors/#allowed-headers).
- [0957a11](https://github.com/static-web-server/static-web-server/commit/0957a11) HTTP methods section. See [docs](https://static-web-server.net/features/http-methods/).

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
- [50974fe](https://github.com/static-web-server/static-web-server/commit/50974fe) Compress WebAssembly (`application/wasm`) files. PR [#84](https://github.com/static-web-server/static-web-server/pull/84) by [@acelot](https://github.com/acelot). See [docs](https://static-web-server.net/features/compression/).
- [70ec60c](https://github.com/static-web-server/static-web-server/commit/70ec60c) Arch Linux [AUR package](https://aur.archlinux.org/packages/static-web-server-bin) support. See [docs](https://static-web-server.net/download-and-install/).

__Refactorings__

- [e109b77](https://github.com/static-web-server/static-web-server/commit/e109b77) Improve startup server error messages providing context.
- [c085147](https://github.com/static-web-server/static-web-server/commit/c085147) Prefer `cfg(unix)` instead of `cfg(not(windows))`.

__Docs__

- [eb482a4](https://github.com/static-web-server/static-web-server/commit/eb482a4) Documentation for Multi-arch Docker images. See [docs](https://static-web-server.net/features/docker/).
- [70ec60c](https://github.com/static-web-server/static-web-server/commit/70ec60c) Documentation for Arch Linux support. See [docs](https://static-web-server.net/download-and-install/).

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

- [3224261](https://github.com/static-web-server/static-web-server/commit/3224261) Configurable grace period support after a `SIGTERM`. PR [#80](https://github.com/static-web-server/static-web-server/pull/80) resolves [#79](https://github.com/static-web-server/static-web-server/issues/79) suggested by [@jtackaberry](https://github.com/jtackaberry). See [docs](https://static-web-server.net/features/graceful-shutdown/#graceful-shutdown) for more details.

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

- [688d1b2](https://github.com/static-web-server/static-web-server/commit/688d1b2) Opt-in sorting by `Name`, `Last Modified` and `File Size` in ascending/descending order via the new `--directory-listing-order` option. More details on [directory listing documentation](https://static-web-server.net/examples/directory-listing/#sorting). PR [#71](https://github.com/static-web-server/static-web-server/pull/71) resolves [#68](https://github.com/static-web-server/static-web-server/issues/68) suggested by [@igoro00](https://github.com/igoro00).

## v2.2.0 - 2021-11-04

__Fixes__

- [c264f2f](https://github.com/static-web-server/static-web-server/commit/c264f2f) Update dependencies (also [e127a1f](https://github.com/static-web-server/static-web-server/commit/e127a1f)).

__Features__

- [0a02da3](https://github.com/static-web-server/static-web-server/commit/0a02da3) [Graceful Shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) support for HTTP/1 - HTTP/2 servers by default. PR [#62](https://github.com/static-web-server/static-web-server/pull/62) resolves [#61](https://github.com/static-web-server/static-web-server/issues/53) suggested by [@pdfrod](https://github.com/pdfrod).

__Refactorings__

- [6f10ef1](https://github.com/static-web-server/static-web-server/commit/6f10ef1) Disable ANSI for tracing logs on Windows in order to display characters correctly.
- [17ceec0](https://github.com/static-web-server/static-web-server/commit/17ceec0) Log Basic Authentication info.

__Docs__

- [b501c40](https://github.com/static-web-server/static-web-server/commit/b501c40) Project Website - [static-web-server.net](https://static-web-server.net). PR [#56](https://github.com/static-web-server/static-web-server/pull/56).

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

- [7265f6b](https://github.com/static-web-server/static-web-server/commit/7265f6b) GitHub Actions as new CI.
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
