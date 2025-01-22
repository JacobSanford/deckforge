use sha3::{Digest, Keccak256};

use crate::crypto::keypair::KeyPair;

#[derive(Clone)]
pub struct Wallet {
    pub pub_key: String,
    pub secret_key: String,
    pub address: String,
}

impl Wallet {
    const ADDRESS_LENGTH: usize = 40;
    const ADDRESS_PREFIX: &'static str = "0x";

    pub fn new() -> Wallet {
        let key_pair = KeyPair::new();
        let pub_key = key_pair.public_key_as_string();
        let secret_key = key_pair.secret_key_as_string();
        let address = Wallet::pub_key_to_wallet_address(&pub_key);
        Wallet {
            pub_key,
            secret_key,
            address,
        }
    }

    pub fn from_keys(pub_key: String, secret_key: String) -> Wallet {
        let address = Wallet::pub_key_to_wallet_address(&pub_key);
        Wallet {
            pub_key,
            secret_key,
            address,
        }
    }

    pub fn from_pem(pem: &str) -> Wallet {
        let (pub_key, secret_key, address) = Wallet::decode_pem(pem);
        Wallet {
            pub_key,
            secret_key,
            address,
        }
    }

    fn pub_key_to_wallet_address(pub_key: &str) -> String {
        let hash = Keccak256::digest(hex::decode(pub_key).unwrap());
        format!(
            "{}{}",
            Wallet::ADDRESS_PREFIX, hex::encode(&hash[12..])
        )
    }

    pub fn to_pem(&self) -> String {
        KeyPair::from_keys(self.pub_key.clone(), self.secret_key.clone()).as_pem()
    }

    fn decode_pem(pem: &str) -> (String, String, String) {
        let key_pair = KeyPair::from_pem(pem);
        let pub_key = key_pair.public_key_as_string();
        let secret_key = key_pair.secret_key_as_string();
        let address = Wallet::pub_key_to_wallet_address(&pub_key);
        (pub_key, secret_key, address)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_basics() {
        let wallet = Wallet::new();
        assert_eq!(wallet.pub_key.len(), 66);
        assert_eq!(wallet.secret_key.len(), 64);
        assert_eq!(wallet.address.len(), 42);
    }

    #[test]
    fn test_wallet_address_generation() {
        let test_public_key = "03e75ab07a0351801c7cc724b01501b3ae43c383cd816cdb28c66386dc670fa98d";
        let test_known_address = "0x8ba82d54332db0c58edc1120a15409aa8cd5f7d9";
        let address = Wallet::pub_key_to_wallet_address(test_public_key);
        assert_eq!(address, test_known_address);
    }

    #[test]
    fn test_wallet_from_pem() {
        let pem_string: &str = "-----BEGIN PUBLIC KEY-----\r\nA+dasHoDUYAcfMcksBUBs65Dw4PNgWzbKMZjhtxnD6mN\r\n-----END PUBLIC KEY-----\r\n\r\n-----BEGIN PRIVATE KEY-----\r\nAlJ3s4doSZ97Cb45mD9sY0IcQHVIVomi1aSGCu+gTtY=\r\n-----END PRIVATE KEY-----\r\n";
        let wallet = Wallet::from_pem(pem_string);
        assert_eq!(wallet.address, "0x8ba82d54332db0c58edc1120a15409aa8cd5f7d9");
        assert_eq!(wallet.pub_key, "03e75ab07a0351801c7cc724b01501b3ae43c383cd816cdb28c66386dc670fa98d");
        assert_eq!(wallet.secret_key, "025277b38768499f7b09be39983f6c63421c4075485689a2d5a4860aefa04ed6");
    }

    #[test]
    fn test_wallet_to_pem() {
        let wallet = Wallet::from_keys(
            "03e75ab07a0351801c7cc724b01501b3ae43c383cd816cdb28c66386dc670fa98d".to_string(),
            "025277b38768499f7b09be39983f6c63421c4075485689a2d5a4860aefa04ed6".to_string(),
        );
        let derived_pem = wallet.to_pem();
        let pem_string: &str = "-----BEGIN PUBLIC KEY-----\r\nA+dasHoDUYAcfMcksBUBs65Dw4PNgWzbKMZjhtxnD6mN\r\n-----END PUBLIC KEY-----\r\n\r\n-----BEGIN PRIVATE KEY-----\r\nAlJ3s4doSZ97Cb45mD9sY0IcQHVIVomi1aSGCu+gTtY=\r\n-----END PRIVATE KEY-----\r\n";
        assert_eq!(derived_pem, pem_string);
    }

}