// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The server CLI options.

use clap::Parser;
use hyper::StatusCode;
use std::{net::IpAddr, path::PathBuf};

#[cfg(feature = "directory-listing")]
use crate::directory_listing::DirListFmt;
use crate::Result;

/// General server configuration available in CLI and config file options.
#[derive(Parser, Debug)]
#[command(author, about, long_about)]
pub struct General {
    #[arg(long, short = 'a', default_value = "::", env = "SERVER_HOST")]
    /// Host address (E.g 127.0.0.1 or ::1)
    pub host: String,

    #[arg(long, short = 'p', default_value = "80", env = "SERVER_PORT")]
    /// Host port
    pub port: u16,

    #[cfg_attr(
        feature = "http2",
        arg(
            long,
            short = 'f',
            env = "SERVER_LISTEN_FD",
            conflicts_with_all(&["host", "port", "https_redirect"])
        )
    )]
    #[cfg_attr(
        not(feature = "http2"),
        arg(
            long,
            short = 'f',
            env = "SERVER_LISTEN_FD",
            conflicts_with_all(&["host", "port"])
        )
    )]
    /// Instead of binding to a TCP port, accept incoming connections to an already-bound TCP
    /// socket listener on the specified file descriptor number (usually zero). Requires that the
    /// parent process (e.g. inetd, launchd, or systemd) binds an address and port on behalf of
    /// static-web-server, before arranging for the resulting file descriptor to be inherited by
    /// static-web-server. Cannot be used in conjunction with the port and host arguments. The
    /// included systemd unit file utilises this feature to increase security by allowing the
    /// static-web-server to be sandboxed more completely.
    pub fd: Option<usize>,

    #[cfg_attr(
        not(target_family = "wasm"),
        arg(
            long,
            short = 'n',
            default_value = "1",
            env = "SERVER_THREADS_MULTIPLIER"
        )
    )]
    #[cfg_attr(
        target_family = "wasm",
        arg(
            long,
            short = 'n',
            default_value = "2",
            env = "SERVER_THREADS_MULTIPLIER"
        )
    )] // We use 2 as the threads multiplier in Wasm, 1 in Native
    /// Number of worker threads multiplier that'll be multiplied by the number of system CPUs
    /// using the formula: `worker threads = number of CPUs * n` where `n` is the value that changes here.
    /// When multiplier value is 0 or 1 then one thread per core is used.
    /// Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side.
    pub threads_multiplier: usize,

    #[cfg_attr(
        not(target_family = "wasm"),
        arg(
            long,
            short = 'b',
            default_value = "512",
            env = "SERVER_MAX_BLOCKING_THREADS"
        )
    )]
    #[cfg_attr(
        target_family = "wasm",
        arg(
            long,
            short = 'b',
            default_value = "20",
            env = "SERVER_MAX_BLOCKING_THREADS"
        )
    )] // We use 20 in Wasm, 512 in Native (default for tokio)
    /// Maximum number of blocking threads
    pub max_blocking_threads: usize,

    #[arg(long, short = 'd', default_value = "./public", env = "SERVER_ROOT")]
    /// Root directory path of static files.
    pub root: PathBuf,

    #[arg(long, default_value = "./50x.html", env = "SERVER_ERROR_PAGE_50X")]
    /// HTML file path for 50x errors. If the path is not specified or simply doesn't exist
    /// then the server will use a generic HTML error message.
    /// If a relative path is used then it will be resolved under the root directory.
    pub page50x: PathBuf,

    #[arg(long, default_value = "./404.html", env = "SERVER_ERROR_PAGE_404")]
    /// HTML file path for 404 errors. If the path is not specified or simply doesn't exist
    /// then the server will use a generic HTML error message.
    /// If a relative path is used then it will be resolved under the root directory.
    pub page404: PathBuf,

    #[cfg(feature = "fallback-page")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fallback-page")))]
    #[arg(long, default_value = "", value_parser = value_parser_pathbuf, env = "SERVER_FALLBACK_PAGE")]
    /// HTML file path that is used for GET requests when the requested path doesn't exist. The fallback page is served with a 200 status code, useful when using client routers. If the path is not specified or simply doesn't exist then this feature will not be active.
    pub page_fallback: PathBuf,

    #[arg(long, short = 'g', default_value = "error", env = "SERVER_LOG_LEVEL")]
    /// Specify a logging level in lower case. Values: error, warn, info, debug or trace
    pub log_level: String,

    #[arg(
        long,
        short = 'c',
        default_value = "",
        env = "SERVER_CORS_ALLOW_ORIGINS"
    )]
    /// Specify an optional CORS list of allowed origin hosts separated by commas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host.
    pub cors_allow_origins: String,

    #[arg(
        long,
        short = 'j',
        default_value = "origin, content-type, authorization",
        env = "SERVER_CORS_ALLOW_HEADERS"
    )]
    /// Specify an optional CORS list of allowed headers separated by commas. Default "origin, content-type". It requires `--cors-allow-origins` to be used along with.
    pub cors_allow_headers: String,

    #[arg(
        long,
        default_value = "origin, content-type",
        env = "SERVER_CORS_EXPOSE_HEADERS"
    )]
    /// Specify an optional CORS list of exposed headers separated by commas. Default "origin, content-type". It requires `--cors-expose-origins` to be used along with.
    pub cors_expose_headers: String,

    #[arg(
        long,
        short = 't',
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_HTTP2_TLS",
    )]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// Enable HTTP/2 with TLS support.
    pub http2: bool,

    #[arg(long, required_if_eq("http2", "true"), env = "SERVER_HTTP2_TLS_CERT")]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// Specify the file path to read the certificate.
    pub http2_tls_cert: Option<PathBuf>,

    #[arg(long, required_if_eq("http2", "true"), env = "SERVER_HTTP2_TLS_KEY")]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// Specify the file path to read the private key.
    pub http2_tls_key: Option<PathBuf>,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        requires_if("true", "http2"),
        env = "SERVER_HTTPS_REDIRECT"
    )]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// Redirect all requests with scheme "http" to "https" for the current server instance. It depends on "http2" to be enabled.
    pub https_redirect: bool,

    #[arg(
        long,
        requires_if("true", "https_redirect"),
        default_value = "localhost",
        env = "SERVER_HTTPS_REDIRECT_HOST"
    )]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// Canonical host name or IP of the HTTPS (HTTPS/2) server. It depends on "https_redirect" to be enabled.
    pub https_redirect_host: String,

    #[arg(
        long,
        requires_if("true", "https_redirect"),
        default_value = "80",
        env = "SERVER_HTTPS_REDIRECT_FROM_PORT"
    )]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// HTTP host port where the redirect server will listen for requests to redirect them to HTTPS. It depends on "https_redirect" to be enabled.
    pub https_redirect_from_port: u16,

    #[arg(
        long,
        requires_if("true", "https_redirect"),
        default_value = "localhost",
        env = "SERVER_HTTPS_REDIRECT_FROM_HOSTS"
    )]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// List of host names or IPs allowed to redirect from. HTTP requests must contain the HTTP 'Host' header and match against this list. It depends on "https_redirect" to be enabled.
    pub https_redirect_from_hosts: String,

    #[arg(long, default_value = "index.html", env = "SERVER_INDEX_FILES")]
    /// List of files that will be used as an index for requests ending with the slash character (‘/’).
    /// Files are checked in the specified order.
    pub index_files: String,

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
    #[arg(
        long,
        short = 'x',
        default_value = "true",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_COMPRESSION",
    )]
    /// Gzip, Deflate, Brotli or Zstd compression on demand determined by the Accept-Encoding header and applied to text-based web file types only.
    pub compression: bool,

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
    #[arg(long, default_value = "default", env = "SERVER_COMPRESSION_LEVEL")]
    /// Compression level to apply for Gzip, Deflate, Brotli or Zstd compression.
    pub compression_level: super::CompressionLevel,

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
    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_COMPRESSION_STATIC",
    )]
    /// Look up the pre-compressed file variant (`.gz`, `.br` or `.zst`) on disk of a requested file and serves it directly if available.
    /// The compression type is determined by the `Accept-Encoding` header.
    pub compression_static: bool,

    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    #[arg(
        long,
        short = 'z',
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_DIRECTORY_LISTING",
    )]
    /// Enable directory listing for all requests ending with the slash character (‘/’).
    pub directory_listing: bool,

    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    #[arg(
        long,
        requires_if("true", "directory_listing"),
        default_value = "6",
        env = "SERVER_DIRECTORY_LISTING_ORDER"
    )]
    /// Specify a default code number to order directory listing entries per `Name`, `Last modified` or `Size` attributes (columns). Code numbers supported: 0 (Name asc), 1 (Name desc), 2 (Last modified asc), 3 (Last modified desc), 4 (Size asc), 5 (Size desc). Default 6 (unordered)
    pub directory_listing_order: u8,

    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    #[arg(
        long,
        value_enum,
        requires_if("true", "directory_listing"),
        default_value = "html",
        env = "SERVER_DIRECTORY_LISTING_FORMAT",
        ignore_case(true)
    )]
    /// Specify a content format for directory listing entries. Formats supported: "html" or "json". Default "html".
    pub directory_listing_format: DirListFmt,

    #[arg(
        long,
        default_value = "false",
        default_value_if("http2", "true", Some("true")),
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_SECURITY_HEADERS",
    )]
    /// Enable security headers by default when HTTP/2 feature is activated.
    /// Headers included: "Strict-Transport-Security: max-age=63072000; includeSubDomains; preload" (2 years max-age),
    /// "X-Frame-Options: DENY" and "Content-Security-Policy: frame-ancestors 'self'".
    pub security_headers: bool,

    #[arg(
        long,
        short = 'e',
        default_value = "true",
        env = "SERVER_CACHE_CONTROL_HEADERS"
    )]
    #[arg(
        long,
        short = 'e',
        default_value = "true",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_CACHE_CONTROL_HEADERS",
    )]
    /// Enable cache control headers for incoming requests based on a set of file types. The file type list can be found on `src/control_headers.rs` file.
    pub cache_control_headers: bool,

    #[cfg(feature = "basic-auth")]
    /// It provides The "Basic" HTTP Authentication scheme using credentials as "user-id:password" pairs. Password must be encoded using the "BCrypt" password-hashing function.
    #[arg(long, default_value = "", env = "SERVER_BASIC_AUTH")]
    pub basic_auth: String,

    #[arg(long, short = 'q', default_value = "0", env = "SERVER_GRACE_PERIOD")]
    /// Defines a grace period in seconds after a `SIGTERM` signal is caught which will delay the server before to shut it down gracefully. The maximum value is 255 seconds.
    pub grace_period: u8,

    #[arg(
        long,
        short = 'w',
        default_value = "./config.toml",
        value_parser = value_parser_pathbuf,
        env = "SERVER_CONFIG_FILE"
    )]
    /// Server TOML configuration file path.
    pub config_file: PathBuf,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_LOG_REMOTE_ADDRESS",
    )]
    /// Log incoming requests information along with its remote address if available using the `info` log level.
    pub log_remote_address: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_LOG_FORWARDED_FOR",
    )]
    /// Log the X-Forwarded-For header for remote IP information
    pub log_forwarded_for: bool,

    #[arg(
        long,
        require_equals(false),
        value_delimiter(','),
        action = clap::ArgAction::Set,
        env = "SERVER_TRUSTED_PROXIES",
    )]
    /// List of IPs to use X-Forwarded-For from. The default is to trust all
    pub trusted_proxies: Vec<IpAddr>,

    #[arg(
        long,
        default_value = "true",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_REDIRECT_TRAILING_SLASH",
    )]
    /// Check for a trailing slash in the requested directory URI and redirect permanently (308) to the same path with a trailing slash suffix if it is missing.
    pub redirect_trailing_slash: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_IGNORE_HIDDEN_FILES",
    )]
    /// Ignore hidden files/directories (dotfiles), preventing them to be served and being included in auto HTML index pages (directory listing).
    pub ignore_hidden_files: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_DISABLE_SYMLINKS",
    )]
    /// Prevent following files or directories if any path name component is a symbolic link.
    pub disable_symlinks: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_HEALTH",
    )]
    /// Add a /health endpoint that doesn't generate any log entry and returns a 200 status code.
    /// This is especially useful with Kubernetes liveness and readiness probes.
    pub health: bool,

    #[cfg(all(unix, feature = "experimental"))]
    #[arg(
        long = "experimental-metrics",
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_EXPERIMENTAL_METRICS",
    )]
    /// Add a /metrics endpoint that returns a Prometheus metrics response.
    pub experimental_metrics: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_MAINTENANCE_MODE"
    )]
    /// Enable the server's maintenance mode functionality.
    pub maintenance_mode: bool,

    #[arg(
        long,
        default_value = "503",
        value_parser = value_parser_status_code,
        requires_if("true", "maintenance_mode"),
        env = "SERVER_MAINTENANCE_MODE_STATUS"
    )]
    /// Provide a custom HTTP status code when entering into maintenance mode. Default 503.
    pub maintenance_mode_status: StatusCode,

    #[arg(
        long,
        default_value = "",
        value_parser = value_parser_pathbuf,
        requires_if("true", "maintenance_mode"),
        env = "SERVER_MAINTENANCE_MODE_FILE"
    )]
    /// Provide a custom maintenance mode HTML file. If not provided then a generic message will be displayed.
    pub maintenance_mode_file: PathBuf,

    //
    // Windows specific arguments and commands
    //
    #[cfg(windows)]
    #[arg(
        long,
        short = 's',
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_WINDOWS_SERVICE",
    )]
    /// Tell the web server to run in a Windows Service context. Note that the `install` subcommand will enable this option automatically.
    pub windows_service: bool,

    // Subcommands
    #[command(subcommand)]
    /// Subcommands for additional maintenance tasks, like installing and uninstalling the SWS Windows Service and generation of completions and man pages
    pub commands: Option<Commands>,

    #[arg(
        long,
        short = 'V',
        default_value = "false",
        default_missing_value("true")
    )]
    #[doc(hidden)]
    /// Print version info and exit.
    pub version: bool,
}

#[derive(Debug, clap::Subcommand)]
/// Subcommands for additional maintenance tasks, like installing and uninstalling the SWS Windows Service and generation of completions and man pages
pub enum Commands {
    /// Install a Windows Service for the web server.
    #[cfg(windows)]
    #[command(name = "install")]
    Install {},

    /// Uninstall the current Windows Service.
    #[cfg(windows)]
    #[command(name = "uninstall")]
    Uninstall {},

    /// Generate man pages and shell completions
    #[command(name = "generate")]
    Generate {
        /// Generate shell completions
        #[arg(long)]
        completions: bool,
        /// Generate man pages
        #[arg(long)]
        man_pages: bool,
        /// Path to write generated artifacts to
        out_dir: PathBuf,
    },
}

fn value_parser_pathbuf(s: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(s))
}

fn value_parser_status_code(s: &str) -> Result<StatusCode, String> {
    match s.parse::<u16>() {
        Ok(code) => StatusCode::from_u16(code).map_err(|err| err.to_string()),
        Err(err) => Err(err.to_string()),
    }
}
