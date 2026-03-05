use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorizedKey {
    pub label: String,
    pub public_key: String,
    pub expiry: DateTime<Utc>,
}

impl AuthorizedKey {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorizedKeys {
    pub keys: Vec<AuthorizedKey>,
}

impl AuthorizedKeys {
    pub fn new() -> Self {
        AuthorizedKeys { keys: Vec::new() }
    }

    pub fn add_key(&mut self, label: String, public_key: String, expiry: DateTime<Utc>) {
        let key = AuthorizedKey {
            label,
            public_key,
            expiry,
        };
        self.keys.push(key);
    }

    pub fn save_to_file(&self, file_path: &str) -> crate::error::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, json)?;
        Ok(())
    }

    pub fn load_from_file(file_path: &str) -> crate::error::Result<Self> {
        let data = fs::read_to_string(file_path)?;
        let keys: AuthorizedKeys = serde_json::from_str(&data)?;
        Ok(keys)
    }

    pub fn is_key_authorized(&self, public_key: &str) -> bool {
        self.keys
            .iter()
            .any(|key| key.public_key == public_key && !key.is_expired())
    }
}
