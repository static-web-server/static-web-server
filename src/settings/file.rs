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
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
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
pub struct Headers {
    pub source: String,
    #[serde(rename(deserialize = "headers"), with = "http_serde::header_map")]
    pub headers: HeaderMap,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u16)]
pub enum RedirectsKind {
    /// Moved Permanently
    Permanent = 301,
    /// Found
    Temporary = 302,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Redirects {
    pub source: String,
    pub destination: String,
    pub kind: RedirectsKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Rewrites {
    pub source: String,
    pub destination: String,
}

/// Advanced server options only available in configuration file mode.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Advanced {
    // Headers
    pub headers: Option<Vec<Headers>>,
    // Rewrites
    pub rewrites: Option<Vec<Rewrites>>,
    // Redirects
    pub redirects: Option<Vec<Redirects>>,
}

/// General server options available in configuration file mode.
/// Note that the `--config-file` option is excluded from itself.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct General {
    // Address & Root dir
    pub host: Option<String>,
    pub port: Option<u16>,
    pub root: Option<PathBuf>,

    // Logging
    pub log_level: Option<LogLevel>,

    // Cache Control headers
    pub cache_control_headers: Option<bool>,

    // Compression
    pub compression: Option<bool>,

    // Check for a pre-compressed file on disk
    pub compression_static: Option<bool>,

    // Error pages
    pub page404: Option<PathBuf>,
    pub page50x: Option<PathBuf>,

    // HTTP/2 + TLS
    pub http2: Option<bool>,
    pub http2_tls_cert: Option<PathBuf>,
    pub http2_tls_key: Option<PathBuf>,

    // Security headers
    pub security_headers: Option<bool>,

    // CORS
    pub cors_allow_origins: Option<String>,
    pub cors_allow_headers: Option<String>,
    pub cors_expose_headers: Option<String>,

    // Directory listing
    pub directory_listing: Option<bool>,
    pub directory_listing_order: Option<u8>,
    pub directory_listing_format: Option<DirListFmt>,

    // Basich Authentication
    pub basic_auth: Option<String>,

    // File descriptor binding
    pub fd: Option<usize>,

    // Worker threads
    pub threads_multiplier: Option<usize>,

    pub grace_period: Option<u8>,

    pub page_fallback: Option<PathBuf>,

    pub log_remote_address: Option<bool>,

    pub redirect_trailing_slash: Option<bool>,

    pub ignore_hidden_files: Option<bool>,

    #[cfg(windows)]
    pub windows_service: Option<bool>,
}

/// Full server configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub general: Option<General>,
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
