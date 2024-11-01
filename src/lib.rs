// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! # Static Web Server (SWS)
//!
//! SWS is a cross-platform, high-performance and asynchronous web server for static files-serving.
//!
//! **Considerations:**
//!
//! This crate was published to make it possible for users to embed SWS with ease instead of working around it via forks.
//! For example, allowing users to turn off default features like `tls` and so on.
//!
//! **However**, because the library is highly coupled with the SWS binary project at this point,
//! users might be limited by the exposed APIs, implementations, missing functionality, or dependencies.
//!
//! In the future, we will eventually plan to make SWS library independent from the binary project.
//!
//! That said, if fine-grained control and flexibility are needed then you could want to look at other alternatives like HTTP libraries or frameworks.
//!
//! **Pre-compile binaries:**
//!
//! This is the official SWS crate.
//! If you are looking for platform pre-compile binaries then
//! take a look at [static-web-server.net/download-and-install](https://static-web-server.net/download-and-install).
//!
//! ## Overview
//!
//! **Static Web Server** (or **`SWS`** abbreviated) is a very small and fast production-ready web server suitable to serve static web files or assets.
//!
//! It is focused on **lightness and easy-to-use** principles while keeping [high performance and safety](https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html) powered by [The Rust Programming Language](https://rust-lang.org).
//!
//! Written on top of [Hyper](https://github.com/hyperium/hyper) and [Tokio](https://github.com/tokio-rs/tokio) runtime. It provides [concurrent and asynchronous networking abilities](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html) as well as the latest HTTP/1 - HTTP/2 implementations.
//!
//! It's cross-platform and available for `Linux`, `macOS`, `Windows` and `FreeBSD` (`x86`/`x86_64`,  `ARM`/`ARM64`) as well as `Docker`.
//!
//! ![static-web-server](https://github.com/static-web-server/static-web-server/assets/1700322/102bef12-1f30-4054-a1bc-30c650d4ffa7)
//!
//! ## Features
//!
//! - Built with [Rust](https://rust-lang.org) which is focused on [safety, speed and concurrency](https://kornel.ski/rust-c-speed).
//! - Memory safe and very reduced CPU and RAM overhead.
//! - Blazing fast static files-serving and asynchronous powered by latest [Hyper](https://github.com/hyperium/hyper/), [Tokio](https://github.com/tokio-rs/tokio) and a set of [awesome crates](https://github.com/static-web-server/static-web-server/blob/master/Cargo.toml).
//! - Single __4MB__ (uncompressed) and fully static binary with no dependencies ([Musl libc](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)). Suitable for running on [any Linux distro](https://en.wikipedia.org/wiki/Linux_distribution) or [Docker container](https://hub.docker.com/r/joseluisq/static-web-server/tags).
//! - Optional GZip, Deflate or Brotli compression for text-based web files only.
//! - Compression on-demand via [Accept-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
//! - [Partial Content Delivery](https://en.wikipedia.org/wiki/Byte_serving) support for byte-serving of large files.
//! - Optional [Cache-Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) headers for assets.
//! - [Termination signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling with [graceful shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) ability and grace period.
//! - [HTTP/2](https://tools.ietf.org/html/rfc7540) and TLS support.
//! - [Security headers](https://web.dev/security-headers/) for HTTP/2 by default.
//! - [HEAD](https://tools.ietf.org/html/rfc7231#section-4.3.2) responses.
//! - Lightweight and configurable logging via [tracing](https://github.com/tokio-rs/tracing) crate.
//! - Customizable number of worker threads.
//! - Optional directory listing.
//! - [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.
//! - Basic HTTP Authentication.
//! - Customizable HTTP response headers for specific file requests via glob patterns.
//! - Fallback pages for 404 errors, useful for Single-page applications.
//! - Run the server as a [Windows Service](https://docs.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc783643(v=ws.10)).
//! - Configurable using CLI arguments, environment variables or a TOML file.
//! - Default and custom error pages.
//! - Custom URL rewrites and redirects via glob patterns.
//! - Support for serving pre-compressed (Gzip/Brotli) files.
//! - First-class [Docker](https://docs.docker.com/get-started/overview/) support. [Scratch](https://hub.docker.com/_/scratch) and latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images.
//! - Ability to accept a socket listener as a file descriptor for use in sandboxing and on-demand applications (E.g [systemd](http://0pointer.de/blog/projects/socket-activation.html)).
//! - Cross-platform. Pre-compiled binaries for Linux, macOS, Windows and FreeBSD (`x86`,`x86_64`,`ARM`,`ARM64`).
//!
//! ## Cargo features
//!
//! When building from the source, all features are enabled by default.
//! However, you can disable just the ones you don't need from the lists below.
//!
//! Feature | Description
//! ---------|------
//! **Default** |
//! `default` | Activates the default features.
//! `all` | Activates all features including the default and experimental ones. E.g. this feature is used when building the SWS binaries.
//! `experimental` | Activates all unstable features.
//! [**HTTP2/TLS**](https://static-web-server.net/features/http2-tls/) |
//! `http2` | Activates the HTTP2 and TLS feature.
//! [**Compression**](https://static-web-server.net/features/compression/) |
//! `compression` | Activates auto-compression and compression static with all supported algorithms.
//! `compression-brotli` | Activates auto-compression/compression static with only the `brotli` algorithm.
//! `compression-deflate` | Activates auto-compression/compression static with only the `deflate` algorithm.
//! `compression-gzip` | Activates auto-compression/compression static with only the `gzip` algorithm.
//! `compression-zstd` | Activates auto-compression/compression static with only the `zstd` algorithm.
//! [**Directory Listing**](https://static-web-server.net/features/directory-listing/) |
//! `directory-listing` | Activates the directory listing feature.
//! [**Basic Authorization**](./features/basic-authentication.md) |
//! `basic-auth` | Activates the Basic HTTP Authorization Schema feature.
//! [**Fallback Page**](./features/error-pages.md#fallback-page-for-use-with-client-routers) |
//! `fallback-page` | Activates the Fallback Page feature.
//!

#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

// Extern crates
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde;

// Public modules
#[macro_use]
pub mod logger;
#[cfg(feature = "basic-auth")]
#[cfg_attr(docsrs, doc(cfg(feature = "basic-auth")))]
pub mod basic_auth;
#[cfg(any(
    feature = "compression",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
    feature = "compression-deflate"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "compression",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd",
        feature = "compression-deflate"
    )))
)]
pub mod compression;
#[cfg(any(
    feature = "compression",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
    feature = "compression-deflate"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "compression",
        feature = "compression-gzip",
        feature = "compression-brotli",
        feature = "compression-zstd",
        feature = "compression-deflate"
    )))
)]
pub mod compression_static;
pub(crate) mod conditional_headers;
pub mod control_headers;
pub mod cors;
pub mod custom_headers;
#[cfg(feature = "directory-listing")]
#[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
pub mod directory_listing;
pub mod error_page;
#[cfg(feature = "fallback-page")]
#[cfg_attr(docsrs, doc(cfg(feature = "fallback-page")))]
pub mod fallback_page;
pub(crate) mod fs;
pub mod handler;
pub(crate) mod headers_ext;
pub(crate) mod health;
#[cfg(feature = "http2")]
#[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
pub mod https_redirect;
pub(crate) mod log_addr;
pub mod maintenance_mode;
#[cfg(feature = "experimental")]
pub(crate) mod mem_cache;
#[cfg(all(unix, feature = "experimental"))]
pub(crate) mod metrics;
pub mod redirects;
pub(crate) mod response;
pub mod rewrites;
pub mod security_headers;
pub mod server;
pub mod service;
pub mod settings;
#[cfg(any(unix, windows))]
#[cfg_attr(docsrs, doc(cfg(any(unix, windows))))]
pub mod signals;
pub mod static_files;
#[cfg(feature = "http2")]
#[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
pub mod tls;
pub mod transport;
pub(crate) mod virtual_hosts;
#[cfg(windows)]
#[cfg_attr(docsrs, doc(cfg(windows)))]
pub mod winservice;
#[macro_use]
pub mod error;

// Private modules
#[doc(hidden)]
mod helpers;
#[doc(hidden)]
pub mod http_ext;
#[doc(hidden)]
pub mod testing;

// Re-exports
pub use error::*;
pub use server::Server;
pub use settings::Settings;
