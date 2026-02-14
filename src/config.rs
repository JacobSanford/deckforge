use serde::Deserialize;
use std::fs;

use crate::error::Result;

#[derive(Deserialize)]
pub struct Config {
    pub data_dir: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
}
