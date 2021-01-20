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

    #[structopt(
        long,
        short = "t",
        default_value = "8",
        env = "SERVER_THREADS_MULTIPLIER"
    )]
    /// Number of worker threads multiplier that'll be multiplied by the number of system CPUs
    /// using the formula: `worker threads = number of CPUs * n` where `n` is the value that changes here.
    /// When multiplier value is 0 or 1 then the `number of CPUs` is used.
    /// Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side.
    pub threads_multiplier: usize,

    #[structopt(long, short = "r", default_value = "./public", env = "SERVER_ROOT")]
    /// Root directory path of static files
    pub root: String,

    #[structopt(long, short = "c", default_value = "gzip", env = "SERVER_COMPRESSION")]
    /// Compression body support for web text-based file types. Values: "gzip", "deflate" or "brotli".
    /// Use an empty value to skip compression.
    pub compression: String,

    #[structopt(long, short = "l", default_value = "error", env = "SERVER_LOG_LEVEL")]
    /// Specify a logging level in lower case.
    pub log_level: String,
}
