use serde::Deserialize;
use std::fs;

use crate::error::Result;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub data_dir: String,
    pub listen_addr: Option<String>,
    pub authorized_keys_path: Option<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&config_content)?;
        config.apply_env_overrides();
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("DECKFORGE_DATA_DIR") {
            self.data_dir = val;
        }
        if let Ok(val) = std::env::var("DECKFORGE_LISTEN_ADDR") {
            self.listen_addr = Some(val);
        }
        if let Ok(val) = std::env::var("DECKFORGE_AUTH_KEYS_PATH") {
            self.authorized_keys_path = Some(val);
        }
    }

    pub fn listen_addr(&self) -> &str {
        self.listen_addr.as_deref().unwrap_or("127.0.0.1:3000")
    }

    pub fn authorized_keys_path(&self) -> &str {
        self.authorized_keys_path.as_deref().unwrap_or("authorized_keys.json")
    }
}
