use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub src: ConfigData,
    pub dst: ConfigData,
}

impl Config {
    pub fn new(config: PathBuf) -> anyhow::Result<Self> {
        let toml_str = std::fs::read_to_string(config)?;
        let config = toml::from_str(&toml_str)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub struct ConfigData {
    pub host: String,
    pub user: String,
    pub password: String,
    pub include: Option<String>,
    pub exclude: Option<String>,
}
