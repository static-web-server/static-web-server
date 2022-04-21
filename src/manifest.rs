//! The server configuration file (Manifest)

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::{helpers, Context, Result};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Headers {
    pub source: String,
    pub headers: Option<Vec<Header>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    // General
    pub host: Option<String>,
    pub port: Option<u8>,
    pub root: Option<String>,

    // Logging
    pub log_level: Option<LogLevel>,

    // Cache Control headers
    pub cache_control_headers: bool,

    // Compression
    pub compression: bool,

    // Error pages
    pub page404: Option<String>,
    pub page50x: Option<String>,

    // HTTP/2 + TLS
    pub http2: bool,
    pub http2_tls_cert: PathBuf,
    pub http2_tls_key: PathBuf,

    // Security headers
    pub security_headers: bool,

    // CORS
    pub cors_allow_origins: String,
    pub cors_allow_headers: String,

    // Directoy listing
    pub directory_listing: bool,
    pub directory_listing_order: u8,

    // Basich Authentication
    pub basic_auth: Option<String>,

    // File descriptor binding
    pub fd: Option<usize>,

    // Worker threads
    pub threads_multiplier: usize,

    pub grace_period: u8,

    pub page_fallback: String,

    // Headers
    #[serde(rename(deserialize = "headers"))]
    pub headers: Option<Vec<Headers>>,
}

/// Read a TOML file from path.
fn read_file(path: &Path) -> Result<toml::Value> {
    let toml_str = helpers::read_file(path).with_context(|| {
        format!(
            "error trying to deserialize configuration \"{}\" file toml.",
            path.display()
        )
    })?;

    let first_error = match toml_str.parse() {
        Ok(res) => return Ok(res),
        Err(err) => err,
    };

    let mut second_parser = toml::de::Deserializer::new(&toml_str);
    second_parser.set_require_newline_after_table(false);
    if let Ok(res) = toml::Value::deserialize(&mut second_parser) {
        let msg = format!(
            "\
TOML file found which contains invalid syntax and will soon not parse
at `{}`.
The TOML spec requires newlines after table definitions (e.g., `[a] b = 1` is
invalid), but this file has a table header which does not have a newline after
it. A newline needs to be added and this warning will soon become a hard error
in the future.",
            path.display()
        );
        println!("{}", &msg);
        return Ok(res);
    }

    let first_error = anyhow::Error::from(first_error);
    Err(first_error.context("could not parse input as TOML format"))
}

/// Detect and read the configuration manifest file by path.
pub fn read_manifest(config_file: &Path) -> Result<Option<Manifest>> {
    // Validate TOML file extension
    let ext = config_file.extension();
    if ext.is_none() || ext.unwrap().is_empty() || ext.unwrap().ne("toml") {
        return Ok(None);
    }

    // TODO: validate minimal TOML file structure needed
    let toml = read_file(config_file).with_context(|| "error reading configuration toml file.")?;
    let mut unused = BTreeSet::new();
    let manifest: Manifest = serde_ignored::deserialize(toml, |path| {
        let mut key = String::new();
        helpers::stringify(&mut key, &path);
        unused.insert(key);
    })
    .with_context(|| "error during configuration toml file deserialization.")?;

    for key in unused {
        println!(
            "Warning: unused configuration manifest key \"{}\" or unsuported.",
            key
        );
    }

    Ok(Some(manifest))
}
