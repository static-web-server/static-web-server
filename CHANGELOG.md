# Static Web Server v3 - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Custom headers for every response.** Added an optional `scope` to `[[advanced.headers]]` rules: `files` (default, static file responses only) or `all`, which also covers redirects, rewrites, `/health`, metrics, rejected CORS requests, Basic auth challenges, maintenance mode and the method check. ([#784](https://github.com/static-web-server/static-web-server/pull/784) by [@frenzzy](https://github.com/frenzzy))

## [v3.0.0-beta.2] - 2026-10-18

This beta adds browser launch, status-based custom headers and automatic directory listing formats, alongside compression fixes and container image updates. It also includes security fixes. The related advisories will be published separately soon.

### Added

- **Open the browser on startup.** Added `--open` to launch the server URL in the default browser and `--open-path` to choose the URL path, with environment variable and TOML support. ([#747](https://github.com/static-web-server/static-web-server/pull/747) by [@shuaixr](https://github.com/shuaixr))
- **Custom headers filtered by response status.** Added an optional `status` list to `[[advanced.headers]]` rules so headers can apply only to selected HTTP status codes. ([#758](https://github.com/static-web-server/static-web-server/pull/758) by [@frenzzy](https://github.com/frenzzy))
- **Automatic directory listing format.** Added `--directory-listing-format=auto` to choose HTML or JSON using the request's `Accept` header and add `Vary: Accept` for caches. ([#738](https://github.com/static-web-server/static-web-server/pull/738) by [@aniketch07](https://github.com/aniketch07))
- **Continuous performance tracking.** Added CodSpeed integration to measure benchmarks in CI while preserving standalone `cargo bench` support. ([#736](https://github.com/static-web-server/static-web-server/pull/736) by [@codspeed\[bot\]](https://github.com/apps/codspeed))
- **Additional header benchmarks.** Added benchmarks for security header injection and `Accept-Encoding` negotiation, and enabled measurement of the `is_allowed` benchmark. ([#736](https://github.com/static-web-server/static-web-server/pull/736) by [@codspeed\[bot\]](https://github.com/apps/codspeed))

### Changed

- **Breaking:** **New default port.** Changed the server and HTTPS redirect listener defaults from `8787` to `8080`, with Docker exposed ports updated to match. ([#733](https://github.com/static-web-server/static-web-server/pull/733) by [@w3lld1](https://github.com/w3lld1))
- **Breaking:** **New default root directory.** SWS now serves the current directory (`.` instead of `./public`) by default. ([#733](https://github.com/static-web-server/static-web-server/pull/733) by [@w3lld1](https://github.com/w3lld1))
- **Dependency updates.** Updated dependencies and switched directory archive generation to `async-tar`'s native Tokio support, removing the async I/O compatibility adapter. ([#753](https://github.com/static-web-server/static-web-server/pull/753), [#780](https://github.com/static-web-server/static-web-server/pull/780) by [@joseluisq](https://github.com/joseluisq))
- **Docker base image updates.** Updated Alpine images to `3.23.6` and Debian images to `13.7-slim`, including FIPS and development variants. ([#776](https://github.com/static-web-server/static-web-server/pull/776) by [@joseluisq](https://github.com/joseluisq))
- **AI-assisted contributor guidance.** Updated `AGENTS.md` and project skills with clearer architecture, configuration, testing and review instructions. ([#775](https://github.com/static-web-server/static-web-server/pull/775) by [@joseluisq](https://github.com/joseluisq))

### Fixed

- **Pre-compressed gzip selection.** Serve `.gz` files only for clients accepting `gzip`, avoiding gzip data incorrectly labeled as `deflate`. ([#762](https://github.com/static-web-server/static-web-server/pull/762) by [@frenzzy](https://github.com/frenzzy))
- **Deflate response format.** On-the-fly `deflate` compression now uses the required zlib format so clients can decode responses correctly. ([#764](https://github.com/static-web-server/static-web-server/pull/764) by [@frenzzy](https://github.com/frenzzy))
- **Partial-content compression.** Skip on-the-fly compression for `206` responses and responses with `Content-Range` so byte ranges remain valid for partial and resumed downloads. ([#768](https://github.com/static-web-server/static-web-server/pull/768) by [@frenzzy](https://github.com/frenzzy))
- **Gzip feature mapping.** Corrected the `compression-gzip` feature to enable the gzip codec instead of deflate. ([#769](https://github.com/static-web-server/static-web-server/pull/769) by [@frenzzy](https://github.com/frenzzy))
- **Compression headers for HEAD requests.** `HEAD` responses now report the same compression headers as `GET`, omitting the uncompressed `Content-Length` when compression is selected. ([#770](https://github.com/static-web-server/static-web-server/pull/770) by [@frenzzy](https://github.com/frenzzy))
- **Zero-quality content preferences.** Exclude `q=0` entries from `Accept-Encoding` and `Accept` negotiation instead of treating them as low-priority choices. ([#778](https://github.com/static-web-server/static-web-server/pull/778) by [@joseluisq](https://github.com/joseluisq))
- **Cache-control for error responses.** File-type cache lifetimes now apply only to `2xx` and `304` responses; other static-file responses use `no-cache` to avoid caching missing assets for a year. ([#757](https://github.com/static-web-server/static-web-server/pull/757) by [@frenzzy](https://github.com/frenzzy))
- **Recursive Rewrite and Redirect matching.** Restored `{**}` capture groups matching nested paths so rules using `$1` work across multiple path segments. ([#734](https://github.com/static-web-server/static-web-server/pull/734) by [@hdimer](https://github.com/hdimer))
- **Header quality-value precision.** Fixed rounding of three-decimal `q` values and capped values at `1` to preserve the correct order during content negotiation. ([#739](https://github.com/static-web-server/static-web-server/pull/739) by [@joseluisq](https://github.com/joseluisq))
- **Fallback page logging.** Suppressed `404` warnings for `GET` requests served by the configured fallback page while keeping warnings for responses that remain errors. ([#737](https://github.com/static-web-server/static-web-server/pull/737) by [@yhuikzdtguioaert](https://github.com/yhuikzdtguioaert))
- **Benchmark dependency fix.** Updated the benchmark crate's SWS dependency to resolve against `v3.0.0-beta.1`. ([#736](https://github.com/static-web-server/static-web-server/pull/736) by [@codspeed\[bot\]](https://github.com/apps/codspeed))
- **NetBSD cross-build fix.** Replaced the unavailable NetBSD 9.4 base archive with the 9.5 archive from NetBSD's CDN and added an archive-host fallback in `Cross.toml`. ([#743](https://github.com/static-web-server/static-web-server/pull/743) by [@joseluisq](https://github.com/joseluisq))

For more details see the [v3.0.0-beta.2 milestone][v300beta2-milestone] and the full changelog [v3.0.0-beta.1...v3.0.0-beta.2][v300beta2-diff].

## [v3.0.0-beta.1] - 2026-07-21

We are excited to announce our first `v3.0.0-beta.1` release for the upcoming major `v3.0.0` which is a significant milestone in the evolution of **SWS** bringing new features, performance improvements, security hardening, among other enhancements.

The release upgrades the HTTP stack to the latest [Hyper v1](https://seanmonstar.com/blog/hyper-v1/), introduces new features like weak ETag validation, structured JSON logging and file logging, stabilizes the in-memory cache, provides FIPS-capable TLS binary support and delivers significant performance and security hardening across all modules. Also, SWS defaults are now more secure out of the box.

Feel free to test this beta release. More betas will follow toward the major `v3.0.0`. Please report any issues or suggestions on the [GitHub Issues](https://github.com/static-web-server/static-web-server/issues).
Your feedback and contributions are appreciated.

Visit our updated website at [static-web-server.net](https://static-web-server.net) for more information, documentation, and examples. Also the [v2 to v3 migration guide](https://static-web-server.net/v3/migration-guide.html).

### Added

- **Weak ETag support via `--etag` option.** Enables browser and CDN revalidation with `If-None-Match` / `If-Match` / `If-Range` support per RFC 7232. Returns `304 Not Modified` for unchanged resources. Integrates with cache-control headers and the in-memory cache. ([#679](https://github.com/static-web-server/static-web-server/pull/679))
- **Structured JSON logging via `--log-format` option.** Logs are now emitted as single-line JSON by default (`--log-format json`). A `--log-format pretty` mode is available for human-readable development output. ([#680](https://github.com/static-web-server/static-web-server/pull/680))
- **File logging via `--log-file` option.** Streams log records to a file on disk in addition to stderr. Uses a non-blocking background writer thread so the request path is never blocked by disk I/O. ([#682](https://github.com/static-web-server/static-web-server/pull/682))
- **Minimum size threshold for dynamic compression.** Responses smaller than `200` bytes bypass dynamic compression, avoiding CPU overhead on tiny payloads. ([#661](https://github.com/static-web-server/static-web-server/pull/661))
- **Backported default text-charset and FIPS 140-validated cryptography features** to the `v3`. ([#658](https://github.com/static-web-server/static-web-server/pull/658))
- **New `--use-relative-root` option.** To instruct SWS to skip the canonicalization of the webroot (`--root`) at startup time and instead, to resolve the webroot at request time (e.g. symlink webroot). ([#717](https://github.com/static-web-server/static-web-server/pull/717))
- **LLM-optimized project files for AI-assisted coding.** Added several LLM-specialized files for working with AI assistants (agents). The `CONTRIBUTORS.md` file has been updated to include a new section for AI-assisted coding contributions. ([#708](https://github.com/static-web-server/static-web-server/pull/708))
- **Increased test cases** for conditional headers, error pages, helpers, HTTP extensions, HTTPS redirect, response handling, and security headers modules. ([#648](https://github.com/static-web-server/static-web-server/pull/648))
- **Improved test coverage** across hardening, byte-range, directory listing, compression, ETag, and TLS modules.

### Changed

- **Breaking:** **Hyper v1 upgrade.** The HTTP stack is now built on Hyper v1, `http` v1 and `http-body-util` crates. Response body handling and server connection wiring have been updated accordingly. ([#647](https://github.com/static-web-server/static-web-server/pull/647))
- **Breaking:** **TLS separated from HTTP/2.** Replaced `--http2`, `--http2-tls-cert` and `--http2-tls-key` with `--tls`, `--tls-cert`, `--tls-key` and a standalone `--http2` flag. SWS now supports HTTP/1, HTTP/1+TLS and HTTP/2+TLS server modes. ([#650](https://github.com/static-web-server/static-web-server/pull/650))
- **Breaking:** **New server defaults.** Default port changed from `80` to `8787` (unprivileged). `--compression-static` and `--security-headers` are now enabled by default. `Cache-Control` defaults to `no-cache` for HTML/JSON and applies 1-year `max-age` only to immutable static assets. New `Referrer-Policy: strict-origin-when-cross-origin` security header. ([#660](https://github.com/static-web-server/static-web-server/pull/660))
- **Breaking:** **Default port consistency across all references.** Updated all Dockerfiles and CLI defaults to consistently reference the new default port `8787`. ([#720](https://github.com/static-web-server/static-web-server/pull/720))
- **Stabilized In-memory Cache.** The experimental in-memory cache advanced feature is now stable. Supports TTL, TTI, LFU admission, LRU eviction, configurable capacity and max file size. Adds `X-Cache` header on cache hits. Byte-range requests are excluded from caching. ([#665](https://github.com/static-web-server/static-web-server/pull/665))
- **Static compression independent of dynamic compression.** The `--compression-static` feature no longer depends on the dynamic compression feature flag and can be used standalone. ([#662](https://github.com/static-web-server/static-web-server/pull/662))
- **Several optimizations across In-memory cache, URL rewrites/redirects, CORS, security and cache-control headers**: such as precompiled *AhoCorasick* automata for placeholder replacement, static `HeaderValue` constants instead of per-response construction, cached CORS validation results, and optimized `Accept-Encoding` quality-value parsing. ([#652](https://github.com/static-web-server/static-web-server/pull/652))
- **Several optimizations for file metadata, static file probes, content-type resolution, and root path canonicalization**. ([#666](https://github.com/static-web-server/static-web-server/pull/666))
- **Directory listing** performance improved with better allocation patterns and reduced per-entry overhead. ([#667](https://github.com/static-web-server/static-web-server/pull/667))
- **Server module restructured** from a single `server.rs` into sub-modules (`http1`, `http1_tls`, `http2`, `redirect`, `listener`, `opts`) for better maintainability. ([#649](https://github.com/static-web-server/static-web-server/pull/649))
- **Extensions-related code restructured** for cleaner module boundaries. ([#664](https://github.com/static-web-server/static-web-server/pull/664))
- **Directory listing download module restructured.** ([#670](https://github.com/static-web-server/static-web-server/pull/670))
- **Updated documentation website** to reflect options, usage and examples, new features, and others. See [static-web-server.net](https://static-web-server.net).

### Removed

- **Breaking:** **Removed `--disable-symlinks` and `--ignore-hidden-files`.** Replaced them with `--follow-symlinks` and `--include-hidden` respectively, both defaulting to `false` (disabled). ([#671](https://github.com/static-web-server/static-web-server/pull/671))

### Fixed

- **Several fixes and improvements** due to Hyper v1 upgrade like (Delayed graceful shutdown [#335](https://github.com/static-web-server/static-web-server/issues/335)) and others. See [Hyper v1 release notes](https://github.com/hyperium/hyper/releases/tag/v1.0.0) for more details.
- **Byte-range suffix detection and normalization** improved with additional edge-case handling. ([#669](https://github.com/static-web-server/static-web-server/pull/669))
- **Windows Ctrl+C deadlock** fixed : Resolved a deadlock chain in the graceful-shutdown signal handling on Windows (v3 only). ([#650](https://github.com/static-web-server/static-web-server/pull/650))
- **URL redirects now preserve the client's query string.** Redirect rules append the original query string to the destination URL in an Apache `QSA` (Query String Append) fashion. Query strings on the client request are no longer silently discarded during redirect processing. ([#709](https://github.com/static-web-server/static-web-server/pull/709))
- **Metrics endpoint authentication reorder.** The metrics endpoint check now runs before CORS pre-processing in the request pipeline, ensuring that basic authentication is properly enforced on the `/metrics` endpoint when both features are enabled. ([#718](https://github.com/static-web-server/static-web-server/pull/718))
- **Pre-compressed variant body truncation.** When compression-static served a pre-compressed variant, the response body was incorrectly sized using the *original* file's metadata. ([#722](https://github.com/static-web-server/static-web-server/pull/722))
- **Removed several potential production panics**, improved error propagation, and increased robustness across multiple modules. ([#668](https://github.com/static-web-server/static-web-server/pull/668))
- **Fixed documentation directory related tests** following the relocation of documentation governance files (`CODE_OF_CONDUCT.md`, `CODE_STYLE.md`, `COMMITS.md`, `PULL_REQUESTS.md`) from the repository root into the `docs/` directory. ([#707](https://github.com/static-web-server/static-web-server/pull/707))

### Security

- **Markdown content negotiation security hardening.** Added security policies for the markdown content negotiation module in a similar way to the static files module. ([#719](https://github.com/static-web-server/static-web-server/pull/719))

[unreleased]: https://github.com/static-web-server/static-web-server/compare/v3.0.0-beta.2...HEAD
[v3.0.0-beta.2]: https://github.com/static-web-server/static-web-server/compare/v3.0.0-beta.1...v3.0.0-beta.2
[v3.0.0-beta.1]: https://github.com/static-web-server/static-web-server/releases/tag/v3.0.0-beta.1
[v300beta2-diff]: https://github.com/static-web-server/static-web-server/compare/v3.0.0-beta.1...v3.0.0-beta.2
[v300beta2-milestone]: https://github.com/static-web-server/static-web-server/milestone/42?closed=1
