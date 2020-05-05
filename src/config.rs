use structopt::StructOpt;

/// A blazing fast static files-serving web server powered by Rust Iron
#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(long, default_value = "my-static-server", env = "SERVER_NAME")]
    /// Name for server
    pub name: String,
    #[structopt(long, default_value = "[::]", env = "SERVER_HOST")]
    /// Host address (E.g 127.0.0.1)
    pub host: String,
    #[structopt(long, default_value = "80", env = "SERVER_PORT")]
    /// Host port
    pub port: u16,
    #[structopt(long, default_value = "./public", env = "SERVER_ROOT")]
    /// Root directory path of static files
    pub root: String,
    #[structopt(long, default_value = "./public/assets", env = "SERVER_ASSETS")]
    /// Assets directory path for add cache headers functionality
    pub assets: String,
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
    #[structopt(long, env = "SERVER_TLS")]
    /// Enables TLS/SSL support.
    pub tls: bool,
    #[structopt(long, default_value = "", env = "SERVER_TLS_PKCS12")]
    /// A cryptographic identity PKCS #12 bundle file path containing a X509 certificate along with its corresponding private key and chain of certificates to a trusted root.
    pub tls_pkcs12: String,
    #[structopt(long, default_value = "", env = "SERVER_TLS_PKCS12_PASSWD")]
    /// A specified password to decrypt the private key.
    pub tls_pkcs12_passwd: String,
    #[structopt(long, default_value = "error", env = "SERVER_LOG_LEVEL")]
    /// Specify a logging level in lower case.
    pub log_level: String,
    #[structopt(long, default_value = "", env = "SERVER_CORS_ALLOW_ORIGINS")]
    /// Specify a CORS list of allowed origin hosts separated by comas with no whitespaces. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host.
    pub cors_allow_origins: String,
}
