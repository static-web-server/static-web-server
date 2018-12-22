#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_assets")]
    pub assets: String,
}

pub fn default_port() -> u16 {
    8018
}

pub fn default_root() -> String {
    "./public".to_string()
}

pub fn default_assets() -> String {
    "./assets".to_string()
}
