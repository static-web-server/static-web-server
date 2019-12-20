use structopt::StructOpt;

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
    #[structopt(long, default_value = "./assets", env = "SERVER_ASSETS")]
    /// Assets directory path for add cache headers functionality
    pub assets: String,
}
