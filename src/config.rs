use structopt::StructOpt;

/// Static Web Server
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
        short = "n",
        default_value = "8",
        env = "SERVER_THREADS_MULTIPLIER"
    )]
    /// Number of worker threads multiplier that'll be multiplied by the number of system CPUs
    /// using the formula: `worker threads = number of CPUs * n` where `n` is the value that changes here.
    /// When multiplier value is 0 or 1 then the `number of CPUs` is used.
    /// Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side.
    pub threads_multiplier: usize,

    #[structopt(long, short = "d", default_value = "./public", env = "SERVER_ROOT")]
    /// Root directory path of static files
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

    #[structopt(long, short = "x", default_value = "gzip", env = "SERVER_COMPRESSION")]
    /// Compression body support for web text-based file types. Values: "gzip" or "brotli".
    /// Use an empty value to skip compression.
    pub compression: String,

    #[structopt(long, short = "g", default_value = "error", env = "SERVER_LOG_LEVEL")]
    /// Specify a logging level in lower case.
    pub log_level: String,

    #[structopt(
        long,
        short = "c",
        default_value = "",
        env = "SERVER_CORS_ALLOW_ORIGINS"
    )]
    /// Specify a optional CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host.
    pub cors_allow_origins: String,

    #[structopt(long, short = "t", env = "SERVER_HTTP2_TLS")]
    /// Enables HTTP/2 with TLS support.
    pub http2: bool,

    #[structopt(long, default_value = "", env = "SERVER_HTTP2_TLS_CERT")]
    /// Specify the file path to read the certificate.
    pub http2_tls_cert: String,

    #[structopt(long, default_value = "", env = "SERVER_HTTP2_TLS_KEY")]
    /// Specify the file path to read the private key.
    pub http2_tls_key: String,
}
