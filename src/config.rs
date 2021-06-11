use structopt::StructOpt;

/// A blazing fast static files-serving web server powered by Rust
#[derive(Debug, StructOpt)]
pub struct Config {
    #[structopt(long, short = "a", default_value = "::", env = "SERVER_HOST")]
    /// Host address (E.g 127.0.0.1 or ::1)
    pub host: String,

    #[structopt(long, short = "p", default_value = "80", env = "SERVER_PORT")]
    /// Host port
    pub port: u16,

    #[structopt(
        long,
        short = "f",
        env = "SERVER_LISTEN_FD",
        conflicts_with_all(&["host", "port"])
    )]
    /// Instead of binding to a TCP port, accept incoming connections to an already-bound TCP
    /// socket listener on the specified file descriptor number (usually zero). Requires that the
    /// parent process (e.g. inetd, launchd, or systemd) binds an address and port on behalf of
    /// static-web-server, before arranging for the resulting file descriptor to be inherited by
    /// static-web-server. Cannot be used in conjunction with the port and host arguments. The
    /// included systemd unit file utilises this feature to increase security by allowing the
    /// static-web-server to be sandboxed more completely.
    pub fd: Option<usize>,

    #[structopt(
        long,
        short = "n",
        default_value = "1",
        env = "SERVER_THREADS_MULTIPLIER"
    )]
    /// Number of worker threads multiplier that'll be multiplied by the number of system CPUs
    /// using the formula: `worker threads = number of CPUs * n` where `n` is the value that changes here.
    /// When multiplier value is 0 or 1 then one thread per core is used.
    /// Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side.
    pub threads_multiplier: usize,

    #[structopt(long, short = "d", default_value = "./public", env = "SERVER_ROOT")]
    /// Root directory path of static files.
    pub root: String,

    #[structopt(
        long,
        default_value = "./public/50x.html",
        env = "SERVER_ERROR_PAGE_50X"
    )]
    /// HTML file path for 50x errors. If path is not specified or simply don't exists then server will use a generic HTML error message.
    pub page50x: String,

    #[structopt(
        long,
        default_value = "./public/404.html",
        env = "SERVER_ERROR_PAGE_404"
    )]
    /// HTML file path for 404 errors. If path is not specified or simply don't exists then server will use a generic HTML error message.
    pub page404: String,

    #[structopt(long, short = "g", default_value = "error", env = "SERVER_LOG_LEVEL")]
    /// Specify a logging level in lower case. Values: error, warn, info, debug or trace
    pub log_level: String,

    #[structopt(
        long,
        short = "c",
        default_value = "",
        env = "SERVER_CORS_ALLOW_ORIGINS"
    )]
    /// Specify an optional CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host.
    pub cors_allow_origins: String,

    #[structopt(
        long,
        short = "t",
        parse(try_from_str),
        default_value = "false",
        env = "SERVER_HTTP2_TLS"
    )]
    /// Enable HTTP/2 with TLS support.
    pub http2: bool,

    #[structopt(
        long,
        required_if("http2", "true"),
        default_value = "",
        env = "SERVER_HTTP2_TLS_CERT"
    )]
    /// Specify the file path to read the certificate.
    pub http2_tls_cert: String,

    #[structopt(
        long,
        required_if("http2", "true"),
        default_value = "",
        env = "SERVER_HTTP2_TLS_KEY"
    )]
    /// Specify the file path to read the private key.
    pub http2_tls_key: String,

    #[structopt(
        long,
        short = "x",
        parse(try_from_str),
        default_value = "true",
        env = "SERVER_COMPRESSION"
    )]
    /// Gzip, Deflate or Brotli compression on demand determined by the Accept-Encoding header and applied to text-based web file types only.
    pub compression: bool,

    #[structopt(
        long,
        short = "z",
        parse(try_from_str),
        default_value = "false",
        env = "SERVER_DIRECTORY_LISTING"
    )]
    /// Enable directory listing for all requests ending with the slash character (‘/’).
    pub directory_listing: bool,

    #[structopt(
        long,
        parse(try_from_str),
        required_if("http2", "true"),
        default_value_if("http2", Some("true"), "true"),
        default_value = "false",
        env = "SERVER_SECURITY_HEADERS"
    )]
    /// Enable security headers by default when HTTP/2 feature is activated.
    /// Headers included: "Strict-Transport-Security: max-age=63072000; includeSubDomains; preload" (2 years max-age),
    /// "X-Frame-Options: DENY", "X-XSS-Protection: 1; mode=block" and "Content-Security-Policy: frame-ancestors 'self'".
    pub security_headers: bool,
}
