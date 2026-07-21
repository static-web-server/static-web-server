# Static Web Server v3 - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v3.0.0-beta.1 - 2026-07-21

We are excited to announce our first `v3.0.0-beta.1` release for the upcoming major `v3.0.0` which is a significant milestone in the evolution of **SWS** bringing new features, performance improvements, security hardening, among other enhancements.

The release upgrades the HTTP stack to the latest [Hyper v1](https://seanmonstar.com/blog/hyper-v1/), introduces new features like weak ETag validation, structured JSON logging and file logging, stabilizes the in-memory cache, provides FIPS-capable TLS binary support and delivers significant performance and security hardening across all modules. Also, SWS defaults are now more secure out of the box.

Feel free to test this beta release. More betas will follow toward the major `v3.0.0`. Please report any issues or suggestions on the [GitHub Issues](https://github.com/static-web-server/static-web-server/issues).
Your feedback and contributions are appreciated.

Visit our updated website at [static-web-server.net](https://static-web-server.net) for more information, documentation, and examples. Also the [v2 to v3 migration guide](https://static-web-server.net/v3/migration-guide.html).

### Breaking Changes

- **Hyper v1 upgrade.** The HTTP stack is now built on Hyper v1, `http` v1 and `http-body-util` crates. Response body handling and server connection wiring have been updated accordingly. ([#647](https://github.com/static-web-server/static-web-server/pull/647))
- **TLS separated from HTTP/2.** Replaced `--http2`, `--http2-tls-cert` and `--http2-tls-key` with `--tls`, `--tls-cert`, `--tls-key` and a standalone `--http2` flag. SWS now supports HTTP/1, HTTP/1+TLS and HTTP/2+TLS server modes. ([#650](https://github.com/static-web-server/static-web-server/pull/650))
- **New server defaults.** Default port changed from `80` to `8787` (unprivileged). `--compression-static` and `--security-headers` are now enabled by default. `Cache-Control` defaults to `no-cache` for HTML/JSON and applies 1-year `max-age` only to immutable static assets. New `Referrer-Policy: strict-origin-when-cross-origin` security header. ([#660](https://github.com/static-web-server/static-web-server/pull/660))
- **Deprecated `--disable-symlinks` and `--ignore-hidden-files`.** are replaced with `--follow-symlinks` and `--include-hidden` respectively, both defaulting to `false` (disabled). ([#671](https://github.com/static-web-server/static-web-server/pull/671))
- **Default port consistency across all references.** Updated all Dockerfiles and CLI defaults to consistently reference the new default port `8787`. ([#720](https://github.com/static-web-server/static-web-server/pull/720))

### New Features

- **Stabilized In-memory Cache.** The experimental in-memory cache advanced feature is now stable. Supports TTL, TTI, LFU admission, LRU eviction, configurable capacity and max file size. Adds `X-Cache` header on cache hits. Byte-range requests are excluded from caching. ([#665](https://github.com/static-web-server/static-web-server/pull/665))
- **Weak ETag support via `--etag` option.** Enables browser and CDN revalidation with `If-None-Match` / `If-Match` / `If-Range` support per RFC 7232. Returns `304 Not Modified` for unchanged resources. Integrates with cache-control headers and the in-memory cache. ([#679](https://github.com/static-web-server/static-web-server/pull/679))
- **Structured JSON logging via `--log-format` option.** Logs are now emitted as single-line JSON by default (`--log-format json`). A `--log-format pretty` mode is available for human-readable development output. ([#680](https://github.com/static-web-server/static-web-server/pull/680))
- **File logging via `--log-file` option.** Streams log records to a file on disk in addition to stderr. Uses a non-blocking background writer thread so the request path is never blocked by disk I/O. ([#682](https://github.com/static-web-server/static-web-server/pull/682))
- **Static compression independent of dynamic compression.** The `--compression-static` feature no longer depends on the dynamic compression feature flag and can be used standalone. ([#662](https://github.com/static-web-server/static-web-server/pull/662))
- **Minimum size threshold for dynamic compression.** Responses smaller than `200` bytes bypass dynamic compression, avoiding CPU overhead on tiny payloads. ([#661](https://github.com/static-web-server/static-web-server/pull/661))
- **Backported default text-charset and FIPS 140-validated cryptography features** to the `v3`. ([#658](https://github.com/static-web-server/static-web-server/pull/658))
- **New `--use-relative-root` option.** To instruct SWS to skip the canonicalization of the webroot (`--root`) at startup time and instead, to resolve the webroot at request time (e.g. symlink webroot). ([#717](https://github.com/static-web-server/static-web-server/pull/717))
- **LLM-optimized project files for AI-assisted coding.** Added several LLM-specialized files for working with AI assistants (agents). The `CONTRIBUTORS.md` file has been updated to include a new section for AI-assisted coding contributions. ([#708](https://github.com/static-web-server/static-web-server/pull/708))

### Performance

- **Several optimizations across In-memory cache, URL rewrites/redirects, CORS, security and cache-control headers**: such as precompiled *AhoCorasick* automata for placeholder replacement, static `HeaderValue` constants instead of per-response construction, cached CORS validation results, and optimized `Accept-Encoding` quality-value parsing. ([#652](https://github.com/static-web-server/static-web-server/pull/652))
- **Several optimizations for file metadata, static file probes, content-type resolution, and root path canonicalization**. ([#666](https://github.com/static-web-server/static-web-server/pull/666))
- **Directory listing** performance improved with better allocation patterns and reduced per-entry overhead. ([#667](https://github.com/static-web-server/static-web-server/pull/667))

### Bug Fixes

- **Several fixes and improvements** due to Hyper v1 upgrade like (Delayed graceful shutdown [#335](https://github.com/static-web-server/static-web-server/issues/335)) and others. See [Hyper v1 release notes](https://github.com/hyperium/hyper/releases/tag/v1.0.0) for more details.
- **Byte-range suffix detection and normalization** improved with additional edge-case handling. ([#669](https://github.com/static-web-server/static-web-server/pull/669))
- **Windows Ctrl+C deadlock** fixed : Resolved a deadlock chain in the graceful-shutdown signal handling on Windows (v3 only). ([#650](https://github.com/static-web-server/static-web-server/pull/650))
- **URL redirects now preserve the client's query string.** Redirect rules append the original query string to the destination URL in an Apache `QSA` (Query String Append) fashion. Query strings on the client request are no longer silently discarded during redirect processing. ([#709](https://github.com/static-web-server/static-web-server/pull/709))
- **Metrics endpoint authentication reorder.** The metrics endpoint check now runs before CORS pre-processing in the request pipeline, ensuring that basic authentication is properly enforced on the `/metrics` endpoint when both features are enabled. ([#718](https://github.com/static-web-server/static-web-server/pull/718))
- **Pre-compressed variant body truncation.** When compression-static served a pre-compressed variant, the response body was incorrectly sized using the *original* file's metadata. ([#722](https://github.com/static-web-server/static-web-server/pull/722))

### Security & Hardening

- **Removed several potential production panics**, improved error propagation, and increased robustness across multiple modules. ([#668](https://github.com/static-web-server/static-web-server/pull/668))
- **Markdown content negotiation security hardening.** Added security policies for the markdown content negotiation module in a similar way to the static files module. ([#719](https://github.com/static-web-server/static-web-server/pull/719))

### Refactoring

- **Server module restructured** from a single `server.rs` into sub-modules (`http1`, `http1_tls`, `http2`, `redirect`, `listener`, `opts`) for better maintainability. ([#649](https://github.com/static-web-server/static-web-server/pull/649))
- **Extensions-related code restructured** for cleaner module boundaries. ([#664](https://github.com/static-web-server/static-web-server/pull/664))
- **Directory listing download module restructured.** ([#670](https://github.com/static-web-server/static-web-server/pull/670))

### Testing

- **Increased test cases** for conditional headers, error pages, helpers, HTTP extensions, HTTPS redirect, response handling, and security headers modules. ([#648](https://github.com/static-web-server/static-web-server/pull/648))
- **Improved test coverage** across hardening, byte-range, directory listing, compression, ETag, and TLS modules.
- **Fixed documentation directory related tests** following the relocation of documentation governance files (`CODE_OF_CONDUCT.md`, `CODE_STYLE.md`, `COMMITS.md`, `PULL_REQUESTS.md`) from the repository root into the `docs/` directory. ([#707](https://github.com/static-web-server/static-web-server/pull/707))

### Documentation

- **Updated documentation website** to reflect options, usage and examples, new features, and others. See [static-web-server.net](https://static-web-server.net).
