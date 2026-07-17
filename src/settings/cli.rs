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

#[cfg(feature = "directory-listing-download")]
use crate::directory_listing::download::DirDownloadFmt;
use crate::logger::LogFormat;

use crate::Result;

/// General server configuration available in CLI and config file options.
#[derive(Parser, Debug)]
#[command(author, about, long_about)]
pub struct General {
    #[arg(long, short = 'a', default_value = "::", env = "SERVER_HOST")]
    /// Host address (E.g 127.0.0.1 or ::1)
    pub host: String,

    #[arg(long, short = 'p', default_value = "8787", env = "SERVER_PORT")]
    /// Host port
    pub port: u16,

    #[cfg_attr(
        feature = "tls",
        arg(
            long,
            short = 'f',
            env = "SERVER_LISTEN_FD",
            conflicts_with_all(&["host", "port", "https_redirect"])
        )
    )]
    #[cfg_attr(
        not(feature = "tls"),
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

    // Unix Domain Socket (UDS) options
    // Mutually exclusive with TCP-based options (host/port/fd) and TLS.
    // Gated to Unix targets; on Windows these flags are not exposed.
    #[cfg(unix)]
    #[cfg_attr(
        feature = "tls",
        arg(
            long,
            env = "SERVER_UNIX_SOCKET",
            conflicts_with_all(&["host", "port", "fd", "tls", "https_redirect"]),
        )
    )]
    #[cfg_attr(
        not(feature = "tls"),
        arg(
            long,
            env = "SERVER_UNIX_SOCKET",
            conflicts_with_all(&["host", "port", "fd"]),
        )
    )]
    /// Bind the server to a Unix Domain Socket (UDS) at the given filesystem path
    /// instead of a TCP host/port. Useful for reverse-proxy setups (e.g. nginx) on the
    /// same host where TCP/IP overhead is undesirable and filesystem-based access
    /// control is preferred. Cannot be combined with `--host`, `--port`, `--fd`, or
    /// TLS-related options. The socket file is removed on a graceful shutdown.
    pub unix_socket: Option<PathBuf>,

    #[cfg(unix)]
    #[arg(
        long,
        env = "SERVER_UNIX_SOCKET_MODE",
        value_parser = parse_octal_mode,
        requires = "unix_socket",
    )]
    /// Filesystem permission bits applied to the Unix socket file after binding,
    /// expressed in octal (e.g. `660`, `0660`, or `0o660`). When omitted the socket
    /// is created with the process umask. Only meaningful together with `--unix-socket`.
    pub unix_socket_mode: Option<u32>,

    #[cfg(unix)]
    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_UNIX_SOCKET_FORCE",
        requires = "unix_socket",
    )]
    /// When `true`, remove an existing socket file at `--unix-socket` before binding.
    /// This is useful when the server was previously killed abruptly and left a stale
    /// socket behind. Defaults to `false` to avoid clobbering an unrelated file.
    pub unix_socket_force: bool,

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

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_USE_RELATIVE_ROOT",
    )]
    /// Resolve the web root directory at request time rather than at startup,
    /// allowing symlinked root directories to be swapped at runtime.
    pub use_relative_root: bool,

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
    /// A HTML file path (not relative to the root) used for GET requests when the requested path doesn't exist. The fallback page is served with a 200 status code, useful when using client routers. If the path doesn't exist then the feature is not activated.
    pub page_fallback: PathBuf,

    #[arg(long, short = 'g', default_value = "error", env = "SERVER_LOG_LEVEL")]
    /// Specify a logging level in lower case. Values: error, warn, info, debug or trace
    pub log_level: String,

    #[arg(
        long,
        value_enum,
        default_value = "json",
        env = "SERVER_LOG_FORMAT",
        ignore_case(true)
    )]
    /// Specify the logging output format. Values: json (structured single-line JSON for production) or pretty (human-readable text for development)
    pub log_format: LogFormat,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        action = clap::ArgAction::Set,
        env = "SERVER_LOG_WITH_ANSI",
    )]
    /// Enable or disable ANSI escape codes for colors and other text formatting of the log output. Only effective when `--log-format pretty` is used.
    pub log_with_ansi: bool,

    #[arg(
        long,
        env = "SERVER_LOG_FILE",
        value_parser = value_parser_pathbuf,
    )]
    /// Optional filesystem path to stream log records to in addition to stderr.
    /// When set, logs are written asynchronously through a background worker
    /// thread (non-blocking I/O), so the request path is never delayed by disk
    /// writes. Missing parent directories are created on startup. ANSI escape
    /// codes are always disabled for file output regardless of
    /// `--log-with-ansi`. The file uses the format selected by
    /// `--log-format` (JSON by default). The file is opened in append mode and
    /// is not rotated by SWS, use an external tool (e.g. `logrotate`) for
    /// rotation.
    pub log_file: Option<PathBuf>,

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
        env = "SERVER_TLS",
    )]
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// Enable TLS/HTTPS support. Requires --tls-cert and --tls-key.
    pub tls: bool,

    #[arg(long, required_if_eq("tls", "true"), env = "SERVER_TLS_CERT")]
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// Specify the file path to the TLS certificate.
    pub tls_cert: Option<PathBuf>,

    #[arg(long, required_if_eq("tls", "true"), env = "SERVER_TLS_KEY")]
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// Specify the file path to the TLS private key.
    pub tls_key: Option<PathBuf>,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_HTTP2",
    )]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    /// Enable HTTP/2 protocol support. Requires TLS to be enabled (--tls).
    pub http2: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_HTTPS_REDIRECT"
    )]
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// Redirect all requests with scheme "http" to "https" for the current server instance. Requires TLS to be enabled (--tls).
    pub https_redirect: bool,

    #[arg(long, default_value = "localhost", env = "SERVER_HTTPS_REDIRECT_HOST")]
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// Canonical host name or IP of the HTTPS server. It depends on "https_redirect" to be enabled.
    pub https_redirect_host: String,

    #[arg(long, default_value = "8787", env = "SERVER_HTTPS_REDIRECT_FROM_PORT")]
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// HTTP host port where the redirect server will listen for requests to redirect them to HTTPS. It depends on "https_redirect" to be enabled.
    pub https_redirect_from_port: u16,

    #[arg(
        long,
        default_value = "localhost",
        env = "SERVER_HTTPS_REDIRECT_FROM_HOSTS"
    )]
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
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

    #[arg(
        long,
        default_value = "true",
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

    #[cfg(feature = "directory-listing-download")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing-download")))]
    #[arg(
        long,
        value_delimiter(','),
        value_enum,
        requires_ifs([
            ("targz", "directory_listing"),
        ]),
        require_equals(true),
        action = clap::ArgAction::Set,
        env = "SERVER_DIRECTORY_LISTING_DOWNLOAD",
        ignore_case(true)
    )]
    /// Specify list of enabled format(s) for directory download. Format supported: `targz`. Default to empty list (disabled).
    pub directory_listing_download: Vec<DirDownloadFmt>,

    #[cfg_attr(
        feature = "tls",
        arg(
            long,
            default_value = "true",
            default_missing_value("true"),
            num_args(0..=1),
            require_equals(false),
            action = clap::ArgAction::Set,
            default_value_if("tls", "true", Some("true")),
            env = "SERVER_SECURITY_HEADERS",
        )
    )]
    #[cfg_attr(
        not(feature = "tls"),
        arg(
            long,
            default_value = "false",
            default_missing_value("true"),
            num_args(0..=1),
            require_equals(false),
            action = clap::ArgAction::Set,
            env = "SERVER_SECURITY_HEADERS",
        )
    )]
    /// Enable security headers by default when TLS feature is activated.
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

    #[arg(
        long,
        default_value = "true",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_ETAG",
    )]
    /// Enable weak `ETag` headers (`W/"<mtime>-<size>"`) and full conditional request handling (`If-None-Match`, `If-Match`, `If-Range`). Composes with `--cache-control-headers`; emits validators on every static-file response so clients can revalidate hot HTML even when long `max-age` is configured elsewhere.
    pub etag: bool,

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
        default_value = "./sws.toml",
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
        env = "SERVER_LOG_X_REAL_IP",
    )]
    /// Log the X-Real-IP header for remote IP information.
    pub log_x_real_ip: bool,

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
        env = "SERVER_INCLUDE_HIDDEN",
    )]
    /// Include hidden files/directories (dotfiles), allowing them to be served and listed in auto HTML index pages (directory listing). Disabled by default; hidden files return `404 Not Found`.
    pub include_hidden: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_FOLLOW_SYMLINKS",
    )]
    /// Follow symbolic links when serving files or directories. Disabled by default; requests whose path contains any symlink component return `403 Forbidden`.
    pub follow_symlinks: bool,

    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_ACCEPT_MARKDOWN",
    )]
    /// Enable markdown content negotiation. When a client sends Accept: text/markdown, serve .md or .html.md files if available.
    pub accept_markdown: bool,

    #[arg(
        long,
        default_value = "true",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_TEXT_CHARSET",
    )]
    /// Set a default `charset=utf-8` parameter on limited set of `text` responses that don't already have one.
    pub text_charset: bool,

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

    #[cfg(feature = "metrics")]
    #[arg(
        long,
        default_value = "false",
        default_missing_value("true"),
        num_args(0..=1),
        require_equals(false),
        action = clap::ArgAction::Set,
        env = "SERVER_METRICS",
    )]
    /// Enable the /metrics endpoint that exposes Prometheus metrics for HTTP requests, connections, and latency.
    pub metrics: bool,

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

/// Parse a Unix file mode given in octal (e.g. `660`, `0660`, `0o660`).
///
/// The parser intentionally rejects decimal/hex values: file permission bits
/// are universally expressed in octal, and accepting other bases would silently
/// produce surprising masks (e.g. `660` parsed as decimal is `0o1224`).
#[cfg(unix)]
fn parse_octal_mode(s: &str) -> Result<u32, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("unix socket mode cannot be empty".to_owned());
    }
    let digits = trimmed
        .strip_prefix("0o")
        .or_else(|| trimmed.strip_prefix("0O"))
        .unwrap_or(trimmed);
    let mode = u32::from_str_radix(digits, 8)
        .map_err(|e| format!("invalid octal unix socket mode '{s}': {e}"))?;
    // Reject values that would set bits outside the standard 12-bit Unix mode
    // space (setuid/setgid/sticky + rwx for u/g/o).
    if mode > 0o7777 {
        return Err(format!(
            "unix socket mode '{s}' exceeds maximum 07777 (12-bit permission mask)"
        ));
    }
    Ok(mode)
}

#[cfg(all(test, unix))]
mod tests {
    use super::parse_octal_mode;

    #[test]
    fn parses_bare_octal_digits() {
        // `660` is the canonical "rw-rw----" mask used for socket files in
        // most reverse-proxy setups; ensure the most common input is accepted.
        assert_eq!(parse_octal_mode("660").unwrap(), 0o660);
    }

    #[test]
    fn parses_leading_zero_and_0o_prefix() {
        // Both Unix-style (`0660`) and Rust-style (`0o660`) are accepted; they
        // must produce identical numeric masks.
        assert_eq!(parse_octal_mode("0660").unwrap(), 0o660);
        assert_eq!(parse_octal_mode("0o660").unwrap(), 0o660);
        assert_eq!(parse_octal_mode("0O660").unwrap(), 0o660);
    }

    #[test]
    fn rejects_non_octal_digits() {
        // `8` and `9` are not octal digits; accepting them silently would
        // produce wrong masks.
        assert!(parse_octal_mode("789").is_err());
    }

    #[test]
    fn rejects_empty_and_whitespace() {
        assert!(parse_octal_mode("").is_err());
        assert!(parse_octal_mode("   ").is_err());
    }

    #[test]
    fn rejects_values_above_07777() {
        // Anything beyond the 12-bit permission space is almost certainly a
        // typo (e.g. typed in decimal).
        assert!(parse_octal_mode("10000").is_err());
    }
}
