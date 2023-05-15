// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The server configuration file options (manifest)

use headers::HeaderMap;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::path::Path;
use std::{collections::BTreeSet, path::PathBuf};

use crate::directory_listing::DirListFmt;
use crate::{helpers, Context, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
/// Log level variants.
pub enum LogLevel {
    /// Error log level.
    Error,
    /// Warn log level.
    Warn,
    /// Info log level.
    Info,
    /// Debug log level.
    Debug,
    /// Trace log level.
    Trace,
}

impl LogLevel {
    /// Get log level name.
    pub fn name(&self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
/// Represents an HTTP headers map.
pub struct Headers {
    /// Header source.
    pub source: String,
    #[serde(rename(deserialize = "headers"), with = "http_serde::header_map")]
    /// headers list.
    pub headers: HeaderMap,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u16)]
/// Represents redirects types.
pub enum RedirectsKind {
    /// Moved Permanently
    Permanent = 301,
    /// Found
    Temporary = 302,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
/// Represents redirects types.
pub struct Redirects {
    /// Source of the redirect.
    pub source: String,
    /// Redirect destination.
    pub destination: String,
    /// Redirect type.
    pub kind: RedirectsKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
/// Represents rewrites types.
pub struct Rewrites {
    /// Source of the rewrite.
    pub source: String,
    /// Rewrite destination.
    pub destination: String,
}

/// Advanced server options only available in configuration file mode.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Advanced {
    /// Headers
    pub headers: Option<Vec<Headers>>,
    /// Rewrites
    pub rewrites: Option<Vec<Rewrites>>,
    /// Redirects
    pub redirects: Option<Vec<Redirects>>,
}

/// General server options available in configuration file mode.
/// Note that the `--config-file` option is excluded from itself.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct General {
    /// Server address.
    pub host: Option<String>,
    /// Server port.
    pub port: Option<u16>,
    /// Root directory path.
    pub root: Option<PathBuf>,

    /// Logging.
    pub log_level: Option<LogLevel>,

    /// Cache Control headers.
    pub cache_control_headers: Option<bool>,

    /// Compression.
    #[cfg(feature = "compression")]
    #[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
    pub compression: Option<bool>,

    /// Check for a pre-compressed file on disk.
    #[cfg(feature = "compression")]
    #[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
    pub compression_static: Option<bool>,

    /// Error 404 pages.
    pub page404: Option<PathBuf>,
    /// Error 50x pages.
    pub page50x: Option<PathBuf>,

    /// HTTP/2 + TLS.
    #[cfg(feature = "http2")]
    pub http2: Option<bool>,
    /// Http2 tls certificate feature.
    #[cfg(feature = "http2")]
    pub http2_tls_cert: Option<PathBuf>,
    /// Http2 tls key feature.
    #[cfg(feature = "http2")]
    pub http2_tls_key: Option<PathBuf>,

    /// Redirect all HTTP requests to HTTPS.
    #[cfg(feature = "http2")]
    pub https_redirect: Option<bool>,
    /// Host port for redirecting HTTP requests to HTTPS.
    #[cfg(feature = "http2")]
    pub https_redirect_port: Option<u16>,

    /// Security headers.
    pub security_headers: Option<bool>,

    /// Cors allow origins feature.
    pub cors_allow_origins: Option<String>,
    /// Cors allow headers feature.
    pub cors_allow_headers: Option<String>,
    /// Cors expose headers feature.
    pub cors_expose_headers: Option<String>,

    /// Directory listing feature.
    pub directory_listing: Option<bool>,
    /// Directory listing order feature.
    pub directory_listing_order: Option<u8>,
    /// Directory listing format feature.
    pub directory_listing_format: Option<DirListFmt>,

    /// Basich Authentication feature.
    pub basic_auth: Option<String>,

    /// File descriptor binding feature.
    pub fd: Option<usize>,

    /// Worker threads.
    pub threads_multiplier: Option<usize>,

    /// Max blocking threads feature.
    pub max_blocking_threads: Option<usize>,

    /// Grace period feature.
    pub grace_period: Option<u8>,

    /// Page fallback feature.
    pub page_fallback: Option<PathBuf>,

    /// Log remote address feature.
    pub log_remote_address: Option<bool>,

    /// Redirect trailing slash feature.
    pub redirect_trailing_slash: Option<bool>,

    /// Ignore hidden files feature.
    pub ignore_hidden_files: Option<bool>,

    #[cfg(windows)]
    /// windows service feature.
    pub windows_service: Option<bool>,
}

/// Full server configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    /// General settings.
    pub general: Option<General>,
    /// Advanced settings.
    pub advanced: Option<Advanced>,
}

impl Settings {
    /// Read and deserialize the server TOML configuration file by path.
    pub fn read(config_file: &Path) -> Result<Settings> {
        // Validate TOML file extension
        let ext = config_file.extension();
        if ext.is_none() || ext.unwrap().is_empty() || ext.unwrap().ne("toml") {
            bail!("configuration file should be in toml format. E.g `config.toml`");
        }

        // TODO: validate minimal TOML file structure needed
        let toml =
            read_toml_file(config_file).with_context(|| "error reading toml configuration file")?;
        let mut unused = BTreeSet::new();
        let manifest: Settings = serde_ignored::deserialize(toml, |path| {
            let mut key = String::new();
            helpers::stringify(&mut key, &path);
            unused.insert(key);
        })
        .with_context(|| "error during toml configuration file deserialization")?;

        for key in unused {
            println!("Warning: unused configuration manifest key \"{key}\" or unsupported");
        }

        Ok(manifest)
    }
}

/// Read and parse a TOML file from an specific path.
fn read_toml_file(path: &Path) -> Result<toml::Value> {
    let toml_str = helpers::read_file(path).with_context(|| {
        format!(
            "error trying to deserialize toml configuration file at \"{}\"",
            path.display()
        )
    })?;
    toml_str
        .parse()
        .map_err(|e| anyhow::Error::from(e).context("could not parse input as TOML"))
}
