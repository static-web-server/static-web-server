extern crate env_logger;
extern crate iron;
extern crate playground_middleware;

#[macro_use]
extern crate serde_derive;
extern crate envy;

use iron::prelude::*;
use playground_middleware::{Cache, GuessContentType, ModifyWith, Prefix, Staticfile};
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_root")]
    root: String,
    #[serde(default = "default_assets")]
    assets: String,
}

fn default_port() -> u16 {
    8018
}

fn default_root() -> String {
    "./public".to_string()
}

fn default_assets() -> String {
    "./assets".to_string()
}

fn main() {
    let config = envy::prefixed("APP_")
        .from_env::<Config>()
        .expect("Unable to parsing config from env");

    env_logger::init().expect("Unable to initialize logger");

    let _address = &format!("{}{}", "[::]:", config.port.to_string());

    let files = Staticfile::new(config.root).expect("Directory to serve not found");
    let mut files = Chain::new(files);

    let one_day = Duration::new(60 * 60 * 24, 0);
    let one_year = Duration::new(60 * 60 * 24 * 365, 0);
    let default_content_type = "text/html"
        .parse()
        .expect("Unable to create default content type");

    files.link_after(ModifyWith::new(Cache::new(one_day)));
    files.link_after(Prefix::new(&[config.assets], Cache::new(one_year)));
    files.link_after(GuessContentType::new(default_content_type));

    let _server = Iron::new(files)
        .http(_address)
        .expect("Unable to start server");

    println!("Server listening at {}", _address);
}
