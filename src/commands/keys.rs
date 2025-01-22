use crate::auth;
use dialoguer::{Input, MultiSelect, Confirm};
use std::sync::Arc;
use chrono::{Duration, Utc};

use crate::crypto::keypair::KeyPair;

/// Command: Generates a new Keypair, writing to a PEM file. Adds the public key to an authorized_keys file.
/// 
/// # Arguments
/// 
/// * `label` - An optional `String` that represents the label for the API key. If not provided, the user will be prompted to enter one.
/// * `expiry` - An optional `String` that represents the expiry date for the API key in ISO 8601 format. If not provided, the user will be prompted to set one.
/// 
/// # Example
/// 
/// ```
/// use crate::commands::keys::generate_key;
/// 
/// // Generate a key with a label and expiry
/// generate_key(Some("my-label".to_string()), Some("2024-12-31T23:59:59Z".to_string())).await;
/// 
/// // Generate a key with user prompts for label and expiry
/// generate_key(None, None).await;
/// ```
pub async fn generate_key(label: Option<String>, expiry: Option<String>) {
    let keypair = Arc::new(KeyPair::new());
    let public_key = hex::encode(keypair.public_key);
    let secret_key = hex::encode(keypair.secret_key[..].to_vec());

    let label = match label {
        Some(l) => l,
        None => Input::new()
            .with_prompt("Enter a label for the API key")
            .interact_text()
            .unwrap(),
    };

    let expiry = match expiry {
        Some(e) => e,
        None => {
            let default_expiry = Utc::now() + Duration::days(99 * 365);
            Input::new()
                .with_prompt("Enter an expiry date for the API key (ISO 8601 format)")
                .default(default_expiry.to_rfc3339())
                .interact_text()
                .unwrap()
        },
    };

    let mut authorized_keys = auth::keys::AuthorizedKeys::load_from_file("authorized_keys.json").unwrap_or_else(|_| auth::keys::AuthorizedKeys::new());
    let is_key_authorized = authorized_keys.is_key_authorized(&public_key);

    if is_key_authorized {
        println!("Key already exists in authorized_keys file.");
    } else {
        let confirm = Confirm::new()
            .with_prompt("Do you want to add this key to the authorized_keys file?")
            .interact()
            .unwrap();

        if confirm {
            authorized_keys.add_key(label.clone(), public_key.clone(), expiry.clone());
            authorized_keys.save_to_file("authorized_keys.json").unwrap();
            println!("Key added to authorized_keys file.");
        } else {
            println!("Key not added to authorized_keys file.");
        }
    }

    println!("Public Key: {}", public_key);
    println!("Secret Key: {}", secret_key);
    println!("Label: {}", label);
    println!("Expiry: {}", expiry);
}
