// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that provides all settings of SWS.
//!

use clap::Parser;
use globset::{Glob, GlobMatcher};
use headers::HeaderMap;
use hyper::StatusCode;
use regex::Regex;
use std::path::{Path, PathBuf};

use crate::{helpers, logger, Context, Result};

pub mod cli;
#[doc(hidden)]
pub mod cli_output;
pub mod file;

pub use cli::Commands;

use cli::General;

#[cfg(feature = "experimental")]
use self::file::MemoryCache;

use self::file::{RedirectsKind, Settings as FileSettings};

#[cfg(any(
    feature = "compression",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
    feature = "compression-deflate"
))]
pub use file::CompressionLevel;

/// The `headers` file options.
pub struct Headers {
    /// Source pattern glob matcher
    pub source: GlobMatcher,
    /// Map of custom HTTP headers
    pub headers: HeaderMap,
}

/// The `Rewrites` file options.
pub struct Rewrites {
    /// Source pattern Regex matcher
    pub source: Regex,
    /// A local file that must exist
    pub destination: String,
    /// Optional redirect type either 301 (Moved Permanently) or 302 (Found).
    pub redirect: Option<RedirectsKind>,
}

/// The `Redirects` file options.
pub struct Redirects {
    /// Optional host to match against an incoming URI host if specified
    pub host: Option<String>,
    /// Source pattern Regex matcher
    pub source: Regex,
    /// A local file that must exist
    pub destination: String,
    /// Redirection type either 301 (Moved Permanently) or 302 (Found)
    pub kind: StatusCode,
}

/// The `VirtualHosts` file options.
pub struct VirtualHosts {
    /// The value to check for in the "Host" header
    pub host: String,
    /// The root directory for this virtual host
    pub root: PathBuf,
}

/// The `advanced` file options.
#[derive(Default)]
pub struct Advanced {
    /// Headers list.
    pub headers: Option<Vec<Headers>>,
    /// Rewrites list.
    pub rewrites: Option<Vec<Rewrites>>,
    /// Redirects list.
    pub redirects: Option<Vec<Redirects>>,
    /// Name-based virtual hosting
    pub virtual_hosts: Option<Vec<VirtualHosts>>,
    #[cfg(feature = "experimental")]
    /// In-memory cache feature (experimental).
    pub memory_cache: Option<MemoryCache>,
}

/// The full server CLI and File options.
pub struct Settings {
    /// General server options
    pub general: General,
    /// Advanced server options
    pub advanced: Option<Advanced>,
}

impl Settings {
    /// Reads CLI/Env and config file options returning the server settings.
    /// It also takes care to initialize the logging system with its level
    /// once the `general` settings are determined.
    pub fn get(log_init: bool) -> Result<Settings> {
        Self::read(log_init, true)
    }

    /// Reads CLI/Env and config file options returning the server settings
    /// without parsing arguments useful for testing.
    pub fn get_unparsed(log_init: bool) -> Result<Settings> {
        Self::read(log_init, false)
    }

    fn read(log_init: bool, parse_args: bool) -> Result<Settings> {
        let opts = if parse_args {
            General::parse()
        } else {
            General::parse_from([""])
        };

        // Define the general CLI/file options
        let version = opts.version;
        let mut host = opts.host;
        let mut port = opts.port;
        let mut root = opts.root;
        let mut log_level = opts.log_level;
        let mut config_file = opts.config_file.clone();
        let mut cache_control_headers = opts.cache_control_headers;

        #[cfg(any(
            feature = "compression",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
            feature = "compression-deflate"
        ))]
        let mut compression = opts.compression;
        #[cfg(any(
            feature = "compression",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
            feature = "compression-deflate"
        ))]
        let mut compression_level = opts.compression_level;
        #[cfg(any(
            feature = "compression",
            feature = "compression-gzip",
            feature = "compression-brotli",
            feature = "compression-zstd",
            feature = "compression-deflate"
        ))]
        let mut compression_static = opts.compression_static;

        let mut page404 = opts.page404;
        let mut page50x = opts.page50x;

        #[cfg(feature = "http2")]
        let mut http2 = opts.http2;
        #[cfg(feature = "http2")]
        let mut http2_tls_cert = opts.http2_tls_cert;
        #[cfg(feature = "http2")]
        let mut http2_tls_key = opts.http2_tls_key;
        #[cfg(feature = "http2")]
        let mut https_redirect = opts.https_redirect;
        #[cfg(feature = "http2")]
        let mut https_redirect_host = opts.https_redirect_host;
        #[cfg(feature = "http2")]
        let mut https_redirect_from_port = opts.https_redirect_from_port;
        #[cfg(feature = "http2")]
        let mut https_redirect_from_hosts = opts.https_redirect_from_hosts;

        let mut security_headers = opts.security_headers;
        let mut cors_allow_origins = opts.cors_allow_origins;
        let mut cors_allow_headers = opts.cors_allow_headers;
        let mut cors_expose_headers = opts.cors_expose_headers;

        #[cfg(feature = "directory-listing")]
        let mut directory_listing = opts.directory_listing;
        #[cfg(feature = "directory-listing")]
        let mut directory_listing_order = opts.directory_listing_order;
        #[cfg(feature = "directory-listing")]
        let mut directory_listing_format = opts.directory_listing_format;

        #[cfg(feature = "basic-auth")]
        let mut basic_auth = opts.basic_auth;

        let mut fd = opts.fd;
        let mut threads_multiplier = opts.threads_multiplier;
        let mut max_blocking_threads = opts.max_blocking_threads;
        let mut grace_period = opts.grace_period;

        #[cfg(feature = "fallback-page")]
        let mut page_fallback = opts.page_fallback;

        let mut log_remote_address = opts.log_remote_address;
        let mut log_forwarded_for = opts.log_forwarded_for;
        let mut trusted_proxies = opts.trusted_proxies;
        let mut redirect_trailing_slash = opts.redirect_trailing_slash;
        let mut ignore_hidden_files = opts.ignore_hidden_files;
        let mut disable_symlinks = opts.disable_symlinks;
        let mut index_files = opts.index_files;
        let mut health = opts.health;

        #[cfg(all(unix, feature = "experimental"))]
        let mut experimental_metrics = opts.experimental_metrics;

        let mut maintenance_mode = opts.maintenance_mode;
        let mut maintenance_mode_status = opts.maintenance_mode_status;
        let mut maintenance_mode_file = opts.maintenance_mode_file;

        // Windows-only options
        #[cfg(windows)]
        let mut windows_service = opts.windows_service;

        // Define the advanced file options
        let mut settings_advanced: Option<Advanced> = None;

        // Handle "config file options" and set them when available
        // NOTE: All config file based options shouldn't be mandatory, therefore `Some()` wrapped
        if let Some((settings, config_file_resolved)) = read_file_settings(&opts.config_file)? {
            config_file = config_file_resolved;

            // File-based "general" options
            let has_general_settings = settings.general.is_some();
            if has_general_settings {
                let general = settings.general.unwrap();

                if let Some(v) = general.host {
                    host = v
                }
                if let Some(v) = general.port {
                    port = v
                }
                if let Some(v) = general.root {
                    root = v
                }
                if let Some(ref v) = general.log_level {
                    log_level = v.name().to_lowercase();
                }
                if let Some(v) = general.cache_control_headers {
                    cache_control_headers = v
                }
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd",
                    feature = "compression-deflate"
                ))]
                if let Some(v) = general.compression {
                    compression = v
                }
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd",
                    feature = "compression-deflate"
                ))]
                if let Some(v) = general.compression_level {
                    compression_level = v
                }
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd",
                    feature = "compression-deflate"
                ))]
                if let Some(v) = general.compression_static {
                    compression_static = v
                }
                if let Some(v) = general.page404 {
                    page404 = v
                }
                if let Some(v) = general.page50x {
                    page50x = v
                }
                #[cfg(feature = "http2")]
                if let Some(v) = general.http2 {
                    http2 = v
                }
                #[cfg(feature = "http2")]
                if let Some(v) = general.http2_tls_cert {
                    http2_tls_cert = Some(v)
                }
                #[cfg(feature = "http2")]
                if let Some(v) = general.http2_tls_key {
                    http2_tls_key = Some(v)
                }
                #[cfg(feature = "http2")]
                if let Some(v) = general.https_redirect {
                    https_redirect = v
                }
                #[cfg(feature = "http2")]
                if let Some(v) = general.https_redirect_host {
                    https_redirect_host = v
                }
                #[cfg(feature = "http2")]
                if let Some(v) = general.https_redirect_from_port {
                    https_redirect_from_port = v
                }
                #[cfg(feature = "http2")]
                if let Some(v) = general.https_redirect_from_hosts {
                    https_redirect_from_hosts = v
                }
                #[cfg(feature = "http2")]
                match general.security_headers {
                    Some(v) => security_headers = v,
                    _ => {
                        if http2 {
                            security_headers = true;
                        }
                    }
                }
                #[cfg(not(feature = "http2"))]
                if let Some(v) = general.security_headers {
                    security_headers = v
                }
                if let Some(ref v) = general.cors_allow_origins {
                    v.clone_into(&mut cors_allow_origins)
                }
                if let Some(ref v) = general.cors_allow_headers {
                    v.clone_into(&mut cors_allow_headers)
                }
                if let Some(ref v) = general.cors_expose_headers {
                    v.clone_into(&mut cors_expose_headers)
                }
                #[cfg(feature = "directory-listing")]
                if let Some(v) = general.directory_listing {
                    directory_listing = v
                }
                #[cfg(feature = "directory-listing")]
                if let Some(v) = general.directory_listing_order {
                    directory_listing_order = v
                }
                #[cfg(feature = "directory-listing")]
                if let Some(v) = general.directory_listing_format {
                    directory_listing_format = v
                }
                #[cfg(feature = "basic-auth")]
                if let Some(ref v) = general.basic_auth {
                    v.clone_into(&mut basic_auth)
                }
                if let Some(v) = general.fd {
                    fd = Some(v)
                }
                if let Some(v) = general.threads_multiplier {
                    threads_multiplier = v
                }
                if let Some(v) = general.max_blocking_threads {
                    max_blocking_threads = v
                }
                if let Some(v) = general.grace_period {
                    grace_period = v
                }
                #[cfg(feature = "fallback-page")]
                if let Some(v) = general.page_fallback {
                    page_fallback = v
                }
                if let Some(v) = general.log_remote_address {
                    log_remote_address = v
                }
                if let Some(v) = general.log_forwarded_for {
                    log_forwarded_for = v
                }
                if let Some(v) = general.trusted_proxies {
                    trusted_proxies = v
                }
                if let Some(v) = general.redirect_trailing_slash {
                    redirect_trailing_slash = v
                }
                if let Some(v) = general.ignore_hidden_files {
                    ignore_hidden_files = v
                }
                if let Some(v) = general.disable_symlinks {
                    disable_symlinks = v
                }
                if let Some(v) = general.health {
                    health = v
                }
                #[cfg(all(unix, feature = "experimental"))]
                if let Some(v) = general.experimental_metrics {
                    experimental_metrics = v
                }
                if let Some(v) = general.index_files {
                    index_files = v
                }
                if let Some(v) = general.maintenance_mode {
                    maintenance_mode = v
                }
                if let Some(v) = general.maintenance_mode_status {
                    maintenance_mode_status =
                        StatusCode::from_u16(v).with_context(|| "invalid HTTP status code")?
                }
                if let Some(v) = general.maintenance_mode_file {
                    maintenance_mode_file = v
                }

                // Windows-only options
                #[cfg(windows)]
                if let Some(v) = general.windows_service {
                    windows_service = v
                }
            }

            // Logging system initialization in config file context
            if log_init {
                logger::init(log_level.as_str())?;
            }

            tracing::debug!("config file read successfully");
            tracing::debug!("config file path provided: {}", opts.config_file.display());
            tracing::debug!("config file path resolved: {}", config_file.display());

            if !has_general_settings {
                server_warn!(
                    "config file empty or no `general` settings found, using default values"
                );
            }

            // File-based "advanced" options
            if let Some(advanced) = settings.advanced {
                // 1. Custom HTTP headers assignment
                let headers_entries = match advanced.headers {
                    Some(headers_entries) => {
                        let mut headers_vec: Vec<Headers> = Vec::new();

                        // Compile a glob pattern for each header sources entry
                        for headers_entry in headers_entries.iter() {
                            let source = Glob::new(&headers_entry.source)
                                .with_context(|| {
                                    format!(
                                        "can not compile glob pattern for header source: {}",
                                        &headers_entry.source
                                    )
                                })?
                                .compile_matcher();

                            headers_vec.push(Headers {
                                source,
                                headers: headers_entry.headers.to_owned(),
                            });
                        }
                        Some(headers_vec)
                    }
                    _ => None,
                };

                // 2. Rewrites assignment
                let rewrites_entries = match advanced.rewrites {
                    Some(rewrites_entries) => {
                        let mut rewrites_vec: Vec<Rewrites> = Vec::new();

                        // Compile a glob pattern for each rewrite sources entry
                        for rewrites_entry in rewrites_entries.iter() {
                            let source = Glob::new(&rewrites_entry.source)
                                .with_context(|| {
                                    format!(
                                        "can not compile glob pattern for rewrite source: {}",
                                        &rewrites_entry.source
                                    )
                                })?
                                .compile_matcher();

                            let pattern = source
                                .glob()
                                .regex()
                                .trim_start_matches("(?-u)")
                                .replace("?:.*", ".*")
                                .replace("?:", "")
                                .replace(".*.*", ".*")
                                .to_owned();
                            tracing::debug!(
                                "url rewrites glob pattern: {}",
                                &rewrites_entry.source
                            );
                            tracing::debug!("url rewrites regex equivalent: {}", pattern);

                            let source = Regex::new(&pattern).with_context(|| {
                                    format!(
                                        "can not compile regex pattern equivalent for rewrite source: {}",
                                        &pattern
                                    )
                                })?;

                            rewrites_vec.push(Rewrites {
                                source,
                                destination: rewrites_entry.destination.to_owned(),
                                redirect: rewrites_entry.redirect.to_owned(),
                            });
                        }
                        Some(rewrites_vec)
                    }
                    _ => None,
                };

                // 3. Redirects assignment
                let redirects_entries = match advanced.redirects {
                    Some(redirects_entries) => {
                        let mut redirects_vec: Vec<Redirects> = Vec::new();

                        // Compile a glob pattern for each redirect sources entry
                        for redirects_entry in redirects_entries.iter() {
                            let source = Glob::new(&redirects_entry.source)
                                .with_context(|| {
                                    format!(
                                        "can not compile glob pattern for redirect source: {}",
                                        &redirects_entry.source
                                    )
                                })?
                                .compile_matcher();

                            let pattern = source
                                .glob()
                                .regex()
                                .trim_start_matches("(?-u)")
                                .replace("?:.*", ".*")
                                .replace("?:", "")
                                .replace(".*.*", ".*")
                                .to_owned();
                            tracing::debug!(
                                "url redirects glob pattern: {}",
                                &redirects_entry.source
                            );
                            tracing::debug!("url redirects regex equivalent: {}", pattern);

                            let source = Regex::new(&pattern).with_context(|| {
                                    format!(
                                        "can not compile regex pattern equivalent for redirect source: {}",
                                        &pattern
                                    )
                                })?;

                            let status_code = redirects_entry.kind.to_owned() as u16;
                            redirects_vec.push(Redirects {
                                host: redirects_entry.host.to_owned(),
                                source,
                                destination: redirects_entry.destination.to_owned(),
                                kind: StatusCode::from_u16(status_code).with_context(|| {
                                    format!("invalid redirect status code: {status_code}")
                                })?,
                            });
                        }
                        Some(redirects_vec)
                    }
                    _ => None,
                };

                // 3. Virtual hosts assignment
                let vhosts_entries = match advanced.virtual_hosts {
                    Some(vhosts_entries) => {
                        let mut vhosts_vec: Vec<VirtualHosts> = Vec::new();

                        for vhosts_entry in vhosts_entries.iter() {
                            if let Some(root) = vhosts_entry.root.to_owned() {
                                // Make sure path is valid
                                let root_dir = helpers::get_valid_dirpath(&root)
                                    .with_context(|| "root directory for virtual host was not found or inaccessible")?;
                                tracing::debug!(
                                    "added virtual host: {} -> {}",
                                    vhosts_entry.host,
                                    root_dir.display()
                                );
                                vhosts_vec.push(VirtualHosts {
                                    host: vhosts_entry.host.to_owned(),
                                    root: root_dir,
                                });
                            }
                        }
                        Some(vhosts_vec)
                    }
                    _ => None,
                };

                settings_advanced = Some(Advanced {
                    headers: headers_entries,
                    rewrites: rewrites_entries,
                    redirects: redirects_entries,
                    virtual_hosts: vhosts_entries,
                    #[cfg(feature = "experimental")]
                    memory_cache: advanced.memory_cache,
                });
            }
        } else if log_init {
            // Logging system initialization on demand
            logger::init(log_level.as_str())?;
        }

        Ok(Settings {
            general: General {
                version,
                host,
                port,
                root,
                log_level,
                config_file,
                cache_control_headers,
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd",
                    feature = "compression-deflate"
                ))]
                compression,
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd",
                    feature = "compression-deflate"
                ))]
                compression_level,
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd",
                    feature = "compression-deflate"
                ))]
                compression_static,
                page404,
                page50x,
                #[cfg(feature = "http2")]
                http2,
                #[cfg(feature = "http2")]
                http2_tls_cert,
                #[cfg(feature = "http2")]
                http2_tls_key,
                #[cfg(feature = "http2")]
                https_redirect,
                #[cfg(feature = "http2")]
                https_redirect_host,
                #[cfg(feature = "http2")]
                https_redirect_from_port,
                #[cfg(feature = "http2")]
                https_redirect_from_hosts,
                security_headers,
                cors_allow_origins,
                cors_allow_headers,
                cors_expose_headers,
                #[cfg(feature = "directory-listing")]
                directory_listing,
                #[cfg(feature = "directory-listing")]
                directory_listing_order,
                #[cfg(feature = "directory-listing")]
                directory_listing_format,
                #[cfg(feature = "basic-auth")]
                basic_auth,
                fd,
                threads_multiplier,
                max_blocking_threads,
                grace_period,
                #[cfg(feature = "fallback-page")]
                page_fallback,
                log_remote_address,
                log_forwarded_for,
                trusted_proxies,
                redirect_trailing_slash,
                ignore_hidden_files,
                disable_symlinks,
                index_files,
                health,
                #[cfg(all(unix, feature = "experimental"))]
                experimental_metrics,
                maintenance_mode,
                maintenance_mode_status,
                maintenance_mode_file,

                // Windows-only options and commands
                #[cfg(windows)]
                windows_service,
                commands: opts.commands,
            },
            advanced: settings_advanced,
        })
    }
}

fn read_file_settings(config_file: &Path) -> Result<Option<(FileSettings, PathBuf)>> {
    if config_file.is_file() {
        let file_path_resolved = config_file
            .canonicalize()
            .with_context(|| "unable to resolve toml config file path")?;

        let settings = FileSettings::read(&file_path_resolved).with_context(|| {
            "unable to read toml config file because has invalid format or unsupported options"
        })?;

        return Ok(Some((settings, file_path_resolved)));
    }
    Ok(None)
}
