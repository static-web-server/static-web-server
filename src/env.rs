#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_assets")]
    pub assets: String,
}

pub fn default_name() -> String {
    "nameless".to_string()
}

pub fn default_host() -> String {
    "[::]".to_string()
}

pub fn default_port() -> u16 {
    80
}

pub fn default_root() -> String {
    "./public".to_string()
}

pub fn default_assets() -> String {
    "./assets".to_string()
}
