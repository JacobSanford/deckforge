//! Crypto module for generating and hashing API keys.
//!
//! This module provides functionality to generate, hash, and verify API keys,
//! using secp256k1 for key generation. 
//!
//! # Functions
//!
//! * `generate_keypair` - Generates a new keypair with hashed public key.
//! * `hash_key` - Hashes a key using Keccak-256.
//! * `validate_public_secret_keypair` - Validates a public and secret keypair.
//! 

// Import necessary types from secp256k1 crate
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use secp256k1::rand::rngs::OsRng;
use pem::{Pem, encode_many, parse_many};
use hex;

pub struct KeyPair {
    pub public_key: [u8; 33],
    pub secret_key: SecretKey,
}

impl KeyPair {
    pub fn validate(&self) -> bool {
        self.validate_pub_priv_match()
    }

    fn validate_pub_priv_match(&self) -> bool {
        let secp = Secp256k1::new();

        let derived_public_key = PublicKey::from_secret_key(&secp, &self.secret_key);
        let derived_public_key_string = hex::encode(derived_public_key.serialize());

        derived_public_key_string == hex::encode(self.public_key)
    }

    pub fn new() -> KeyPair {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let pub_key = public_key.serialize();

        KeyPair {
            public_key: pub_key,
            secret_key,
        }
    }

    pub fn as_pem(&self) -> String {
        let public_key_vec = self.public_key.to_vec();
        let secret_key_vec = self.secret_key[..].to_vec();
        let pem = vec![
            Pem::new("PUBLIC KEY", public_key_vec),
            Pem::new("PRIVATE KEY", secret_key_vec),
        ];
        encode_many(&pem)
    }

    pub fn from_keys(public_key: String, secret_key: String) -> KeyPair {
        let public_key = hex::decode(public_key).unwrap();
        let secret_key = SecretKey::from_slice(&hex::decode(secret_key).unwrap()).unwrap();

        KeyPair {
            public_key: public_key.try_into().unwrap(),
            secret_key,
        }
    }

    pub fn from_pem(pem: &str) -> KeyPair {
        let pems = parse_many(pem.as_bytes()).unwrap();
        let public_key = pems[0].contents();
        let secret_key = pems[1].contents();
        let secret_key = SecretKey::from_slice(&secret_key).unwrap();

        KeyPair {
            public_key: public_key.to_vec().try_into().unwrap(),
            secret_key,
        }
    }

    pub fn secret_key_as_string(&self) -> String {
        self.secret_key.display_secret().to_string()
    }

    pub fn public_key_as_string(&self) -> String {
        hex::encode(self.public_key)
    }

}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = KeyPair::new();
        assert_eq!(keypair.validate(), true);
    }

    #[test]
    fn test_from_to_pem() {
        let keypair = KeyPair::new();
        let original_secret_key = keypair.secret_key_as_string();
        let original_public_key = keypair.public_key_as_string();
        let pem = keypair.as_pem();
        let keypair_from_pem = KeyPair::from_pem(&pem);
        assert_eq!(keypair_from_pem.validate(), true);
        assert_eq!(keypair_from_pem.secret_key_as_string(), original_secret_key);
        assert_eq!(keypair_from_pem.public_key_as_string(), original_public_key);
    }
}
