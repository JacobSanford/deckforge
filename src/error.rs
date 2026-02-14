use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeckForgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("Secp256k1 error: {0}")]
    Secp256k1(#[from] secp256k1::Error),

    #[error("PEM error: {0}")]
    Pem(#[from] pem::PemError),

    #[error("Series '{id}' has already been released")]
    AlreadyReleased { id: String },

    #[error("Series '{id}' not found")]
    SeriesNotFound { id: String },

    #[error("No series releases found")]
    NoReleasesFound,

    #[error("Blockchain file not found: {path}")]
    BlockchainNotFound { path: String },

    #[error("Blockchain is empty (no genesis block)")]
    EmptyChain,

    #[error("Validation failed: {reason}")]
    Validation { reason: String },

    #[error("Invalid public key length: expected 33 bytes, got {len}")]
    InvalidPublicKeyLength { len: usize },
}

pub type Result<T> = std::result::Result<T, DeckForgeError>;
