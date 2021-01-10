use structopt::StructOpt;

/// Static Web Server
#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(long, short = "s", default_value = "::", env = "SERVER_HOST")]
    /// Host address (E.g 127.0.0.1 or ::1)
    pub host: String,

    #[structopt(long, short = "p", default_value = "80", env = "SERVER_PORT")]
    /// Host port
    pub port: u16,

    #[structopt(long, short = "r", default_value = "./public", env = "SERVER_ROOT")]
    /// Root directory path of static files
    pub root: String,

    #[structopt(long, short = "c", default_value = "gzip", env = "SERVER_COMPRESSION")]
    /// Compression body support for text-based files. Values: "gzip", "deflate" or "brotli"
    pub compression: String,

    #[structopt(long, short = "l", default_value = "error", env = "SERVER_LOG_LEVEL")]
    /// Specify a logging level in lower case.
    pub log_level: String,
}
