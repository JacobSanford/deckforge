use chrono::{DateTime, Duration, Utc};
use dialoguer::{Confirm, Input};

use crate::auth;
use crate::config::Config;
use crate::crypto::keypair::KeyPair;
use crate::error::{DeckForgeError, Result};

/// Command: Generates a new Keypair, writing to a PEM file.
/// Adds the public key to an authorized_keys file.
pub fn generate_key(label: Option<String>, expiry: Option<String>, config: &Config) -> Result<()> {
    let keypair = KeyPair::new();
    let public_key = hex::encode(keypair.public_key);
    let secret_key = hex::encode(&keypair.secret_key[..]);

    let label = match label {
        Some(l) => l,
        None => Input::new()
            .with_prompt("Enter a label for the API key")
            .interact_text()
            .map_err(|e| DeckForgeError::Dialoguer(e.to_string()))?,
    };

    let default_expiry = Utc::now() + Duration::days(365);
    let expiry: DateTime<Utc> = match expiry {
        Some(e) => e.parse::<DateTime<Utc>>()
            .map_err(|e| DeckForgeError::Chrono(e.to_string()))?,
        None => {
            let expiry_str: String = Input::new()
                .with_prompt("Enter an expiry date for the API key (ISO 8601 format)")
                .default(default_expiry.to_rfc3339())
                .interact_text()
                .map_err(|e| DeckForgeError::Dialoguer(e.to_string()))?;
            expiry_str.parse::<DateTime<Utc>>()
                .map_err(|e| DeckForgeError::Chrono(e.to_string()))?
        }
    };

    let auth_keys_path = config.authorized_keys_path();
    let mut authorized_keys = auth::keys::AuthorizedKeys::load_from_file(auth_keys_path)
        .unwrap_or_else(|_| auth::keys::AuthorizedKeys::new());

    if authorized_keys.is_key_authorized(&public_key) {
        tracing::warn!("Key already exists in authorized_keys file.");
    } else {
        let confirm = Confirm::new()
            .with_prompt("Do you want to add this key to the authorized_keys file?")
            .interact()
            .map_err(|e| DeckForgeError::Dialoguer(e.to_string()))?;

        if confirm {
            authorized_keys.add_key(label.clone(), public_key.clone(), expiry);
            authorized_keys.save_to_file(auth_keys_path)?;
            tracing::info!("Key added to authorized_keys file.");
        } else {
            tracing::info!("Key not added to authorized_keys file.");
        }
    }

    println!("Public Key: {}", public_key);
    println!("Secret Key: {}", secret_key);
    println!("Label: {}", label);
    println!("Expiry: {}", expiry.to_rfc3339());
    Ok(())
}
