use secp256k1::{PublicKey, Secp256k1, SecretKey};
use secp256k1::rand::rngs::OsRng;
use pem::{Pem, encode_many, parse_many};

use crate::error::{DeckForgeError, Result};

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
        derived_public_key.serialize() == self.public_key
    }

    pub fn new() -> KeyPair {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        KeyPair {
            public_key: public_key.serialize(),
            secret_key,
        }
    }

    pub fn as_pem(&self) -> String {
        let pem = vec![
            Pem::new("PUBLIC KEY", self.public_key.to_vec()),
            Pem::new("PRIVATE KEY", self.secret_key[..].to_vec()),
        ];
        encode_many(&pem)
    }

    pub fn from_keys(public_key: &str, secret_key: &str) -> Result<KeyPair> {
        let pub_bytes = hex::decode(public_key)?;
        let pub_array: [u8; 33] = pub_bytes.try_into().map_err(|v: Vec<u8>| {
            DeckForgeError::InvalidPublicKeyLength { len: v.len() }
        })?;
        let secret = SecretKey::from_slice(&hex::decode(secret_key)?)?;

        Ok(KeyPair {
            public_key: pub_array,
            secret_key: secret,
        })
    }

    pub fn from_pem(pem: &str) -> Result<KeyPair> {
        let pems = parse_many(pem.as_bytes())?;
        let public_key = pems.first().ok_or(
            DeckForgeError::Pem(pem::PemError::MalformedFraming)
        )?.contents().to_vec();
        let secret_key_bytes = pems.get(1).ok_or(
            DeckForgeError::Pem(pem::PemError::MalformedFraming)
        )?.contents();
        let secret_key = SecretKey::from_slice(secret_key_bytes)?;
        let pub_array: [u8; 33] = public_key.try_into().map_err(|v: Vec<u8>| {
            DeckForgeError::InvalidPublicKeyLength { len: v.len() }
        })?;

        Ok(KeyPair {
            public_key: pub_array,
            secret_key,
        })
    }

    pub fn secret_key_as_string(&self) -> String {
        self.secret_key.display_secret().to_string()
    }

    pub fn public_key_as_string(&self) -> String {
        hex::encode(self.public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = KeyPair::new();
        assert!(keypair.validate());
    }

    #[test]
    fn test_from_to_pem() {
        let keypair = KeyPair::new();
        let original_secret_key = keypair.secret_key_as_string();
        let original_public_key = keypair.public_key_as_string();
        let pem = keypair.as_pem();
        let keypair_from_pem = KeyPair::from_pem(&pem).unwrap();
        assert!(keypair_from_pem.validate());
        assert_eq!(keypair_from_pem.secret_key_as_string(), original_secret_key);
        assert_eq!(keypair_from_pem.public_key_as_string(), original_public_key);
    }
}
