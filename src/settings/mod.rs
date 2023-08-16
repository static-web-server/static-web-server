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
use std::path::PathBuf;

use crate::{helpers, logger, Context, Result};

pub mod cli;
pub mod file;

#[cfg(windows)]
pub use cli::Commands;

use cli::General;

use self::file::{RedirectsKind, Settings as FileSettings};

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
    /// Source pattern Regex matcher
    pub source: Regex,
    /// A local file that must exist
    pub destination: String,
    /// Redirection type either 301 (Moved Permanently) or 302 (Found)
    pub kind: StatusCode,
}

/// The `Scripts` file options.
pub struct Scripts {
    /// Source pattern Regex matcher
    pub source: Regex,
    /// A local file that must exist
    pub destination: String,
}

/// The `VirtualHosts` file options.
pub struct VirtualHosts {
    /// The value to check for in the "Host" header
    pub host: String,
    /// The root directory for this virtual host
    pub root: PathBuf,
}

/// The `advanced` file options.
pub struct Advanced {
    /// Headers list.
    pub headers: Option<Vec<Headers>>,
    /// Rewrites list.
    pub rewrites: Option<Vec<Rewrites>>,
    /// Redirects list.
    pub redirects: Option<Vec<Redirects>>,
    /// Scripts list.
    pub scripts: Option<Vec<Scripts>>,
    /// Name-based virtual hosting
    pub virtual_hosts: Option<Vec<VirtualHosts>>,
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
        let opts = General::parse();

        // Define the general CLI/file options
        let mut host = opts.host;
        let mut port = opts.port;
        let mut root = opts.root;
        let mut log_level = opts.log_level;
        let mut config_file = opts.config_file.clone();
        let mut cache_control_headers = opts.cache_control_headers;

        #[cfg(feature = "compression")]
        let mut compression = opts.compression;
        #[cfg(feature = "compression")]
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
        let mut redirect_trailing_slash = opts.redirect_trailing_slash;
        let mut ignore_hidden_files = opts.ignore_hidden_files;
        let mut health = opts.health;

        // Windows-only options
        #[cfg(windows)]
        let mut windows_service = opts.windows_service;

        // Define the advanced file options
        let mut settings_advanced: Option<Advanced> = None;

        // Handle "config file options" and set them when available
        // NOTE: All config file based options shouldn't be mandatory, therefore `Some()` wrapped
        if let Some((settings, path_resolved)) = get_file_settings(opts.config_file)? {
            config_file = Some(path_resolved);

            // File-based "general" options
            if let Some(general) = settings.general {
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
                #[cfg(feature = "compression")]
                if let Some(v) = general.compression {
                    compression = v
                }
                #[cfg(feature = "compression")]
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
                    cors_allow_origins = v.to_owned()
                }
                if let Some(ref v) = general.cors_allow_headers {
                    cors_allow_headers = v.to_owned()
                }
                if let Some(ref v) = general.cors_expose_headers {
                    cors_expose_headers = v.to_owned()
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
                    basic_auth = v.to_owned()
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
                if let Some(v) = general.redirect_trailing_slash {
                    redirect_trailing_slash = v
                }
                if let Some(v) = general.ignore_hidden_files {
                    ignore_hidden_files = v
                }
                if let Some(v) = general.health {
                    health = v
                }

                // Windows-only options
                #[cfg(windows)]
                if let Some(v) = general.windows_service {
                    windows_service = v
                }
            }

            // Logging system initialization
            if log_init {
                logger::init(log_level.as_str())?;
            }
            tracing::debug!("toml configuration file read successfully");

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

                            let pattern =
                                source.glob().regex().trim_start_matches("(?-u)").to_owned();
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

                            let pattern =
                                source.glob().regex().trim_start_matches("(?-u)").to_owned();
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

                // 4. Scripts assignment
                let scripts_entries = match advanced.scripts {
                    Some(scripts_entries) => {
                        let mut scripts_vec: Vec<Scripts> = Vec::new();

                        // Compile a glob pattern for each script sources entry
                        for scripts_entry in scripts_entries.iter() {
                            let source = Glob::new(&scripts_entry.source)
                                .with_context(|| {
                                    format!(
                                        "can not compile glob pattern for script source: {}",
                                        &scripts_entry.source
                                    )
                                })?
                                .compile_matcher();

                            let pattern =
                                source.glob().regex().trim_start_matches("(?-u)").to_owned();
                            tracing::debug!("url script glob pattern: {}", &scripts_entry.source);
                            tracing::debug!("url script regex equivalent: {}", pattern);

                            let source = Regex::new(&pattern).with_context(|| {
                                    format!(
                                        "can not compile regex pattern equivalent for script source: {}",
                                        &pattern
                                    )
                                })?;

                            scripts_vec.push(Scripts {
                                source,
                                destination: scripts_entry.destination.to_owned(),
                            });
                        }
                        Some(scripts_vec)
                    }
                    _ => None,
                };

                // 5. Virtual hosts assignment
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
                    scripts: scripts_entries,
                    virtual_hosts: vhosts_entries,
                });
            }
        } else if log_init {
            // Logging system initialization
            logger::init(log_level.as_str())?;
        }

        Ok(Settings {
            general: General {
                host,
                port,
                root,
                log_level,
                config_file,
                cache_control_headers,
                #[cfg(feature = "compression")]
                compression,
                #[cfg(feature = "compression")]
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
                redirect_trailing_slash,
                ignore_hidden_files,
                health,

                // Windows-only options and commands
                #[cfg(windows)]
                windows_service,
                #[cfg(windows)]
                commands: opts.commands,
            },
            advanced: settings_advanced,
        })
    }
}

fn get_file_settings(file_path_opt: Option<PathBuf>) -> Result<Option<(FileSettings, PathBuf)>> {
    if let Some(ref file_path) = file_path_opt {
        if file_path.is_file() {
            let file_path_resolved = file_path
                .canonicalize()
                .with_context(|| "error resolving toml config file path")?;

            let settings = FileSettings::read(&file_path_resolved).with_context(|| {
                "can not read toml config file because has invalid or unsupported format/options"
            })?;

            return Ok(Some((settings, file_path_resolved)));
        }
    }
    Ok(None)
}
