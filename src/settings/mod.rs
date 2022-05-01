use globset::{Glob, GlobMatcher};
use headers::HeaderMap;
use structopt::StructOpt;

use crate::{Context, Result};

mod cli;
pub mod file;

use cli::General;

/// The `headers` file options.
pub struct Headers {
    /// Source pattern glob matcher
    pub source: GlobMatcher,
    /// Map of custom HTTP headers
    pub headers: HeaderMap,
}

/// The `advanced` file options.
pub struct Advanced {
    pub headers: Option<Vec<Headers>>,
}

/// The full server CLI and File options.
pub struct Settings {
    /// General server options
    pub general: General,
    /// Advanced server options
    pub advanced: Option<Advanced>,
}

impl Settings {
    /// Handles CLI and config file options and converging them into one.
    pub fn get() -> Result<Settings> {
        let opts = General::from_args();

        // Define the general CLI/file options
        let mut host = opts.host.to_owned();
        let mut port = opts.port;
        let mut root = opts.root.to_owned();
        let mut log_level = opts.log_level.to_owned();
        let mut config_file = opts.config_file.clone();
        let mut cache_control_headers = opts.cache_control_headers;
        let mut compression = opts.compression;
        let mut page404 = opts.page404.to_owned();
        let mut page50x = opts.page50x.to_owned();
        let mut http2 = opts.http2;
        let mut http2_tls_cert = opts.http2_tls_cert.to_owned();
        let mut http2_tls_key = opts.http2_tls_key.to_owned();
        let mut security_headers = opts.security_headers;
        let mut cors_allow_origins = opts.cors_allow_origins.to_owned();
        let mut cors_allow_headers = opts.cors_allow_headers.to_owned();
        let mut directory_listing = opts.directory_listing;
        let mut directory_listing_order = opts.directory_listing_order;
        let mut basic_auth = opts.basic_auth.to_owned();
        let mut fd = opts.fd;
        let mut threads_multiplier = opts.threads_multiplier;
        let mut grace_period = opts.grace_period;
        let mut page_fallback = opts.page_fallback.to_owned();

        // Define the advanced file options
        let mut settings_advanced: Option<Advanced> = None;

        // Handle "config file options" and set them when available
        // NOTE: All config file based options shouldn't be mandatory, therefore `Some()` wrapped
        if let Some(ref p) = opts.config_file {
            if p.is_file() {
                let path_resolved = p
                    .canonicalize()
                    .with_context(|| "error resolving toml config file path")?;

                let settings = file::Settings::read(&path_resolved)
                    .with_context(|| {
                        "can not read toml config file because has invalid or unsupported format/options"
                    })?;

                config_file = Some(path_resolved);

                // Assign the corresponding file option values
                if let Some(general) = settings.general {
                    if let Some(ref v) = general.host {
                        host = v.to_owned()
                    }
                    if let Some(v) = general.port {
                        port = v
                    }
                    if let Some(ref v) = general.root {
                        root = v.to_owned()
                    }
                    if let Some(ref v) = general.log_level {
                        log_level = v.name().to_lowercase();
                    }
                    if let Some(v) = general.cache_control_headers {
                        cache_control_headers = v
                    }
                    if let Some(v) = general.compression {
                        compression = v
                    }
                    if let Some(ref v) = general.page404 {
                        page404 = v.to_owned()
                    }
                    if let Some(ref v) = general.page50x {
                        page50x = v.to_owned()
                    }
                    if let Some(v) = general.http2 {
                        http2 = v
                    }
                    if let Some(ref v) = general.http2_tls_cert {
                        http2_tls_cert = v.to_owned()
                    }
                    if let Some(ref v) = general.http2_tls_key {
                        http2_tls_key = v.to_owned()
                    }
                    if let Some(v) = general.security_headers {
                        security_headers = v
                    }
                    if let Some(ref v) = general.cors_allow_origins {
                        cors_allow_origins = v.to_owned()
                    }
                    if let Some(ref v) = general.cors_allow_headers {
                        cors_allow_headers = v.to_owned()
                    }
                    if let Some(v) = general.directory_listing {
                        directory_listing = v
                    }
                    if let Some(v) = general.directory_listing_order {
                        directory_listing_order = v
                    }
                    if let Some(ref v) = general.basic_auth {
                        basic_auth = v.to_owned()
                    }
                    if let Some(v) = general.fd {
                        fd = Some(v)
                    }
                    if let Some(v) = general.threads_multiplier {
                        threads_multiplier = v
                    }
                    if let Some(v) = general.grace_period {
                        grace_period = v
                    }
                    if let Some(ref v) = general.page_fallback {
                        page_fallback = v.to_owned()
                    }
                }

                // Prepare the "advanced" options
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

                    settings_advanced = Some(Advanced {
                        headers: headers_entries,
                    });
                }
            }
        }

        Ok(Settings {
            general: General {
                host,
                port,
                root,
                log_level,
                config_file,
                cache_control_headers,
                compression,
                page404,
                page50x,
                http2,
                http2_tls_cert,
                http2_tls_key,
                security_headers,
                cors_allow_origins,
                cors_allow_headers,
                directory_listing,
                directory_listing_order,
                basic_auth,
                fd,
                threads_multiplier,
                grace_period,
                page_fallback,
            },
            advanced: settings_advanced,
        })
    }
}
