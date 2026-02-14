use dialoguer::{Confirm, Input};

use crate::auth;
use crate::crypto::keypair::KeyPair;

/// Command: Generates a new Keypair, writing to a PEM file.
/// Adds the public key to an authorized_keys file.
pub fn generate_key(label: Option<String>, expiry: Option<String>) -> anyhow::Result<()> {
    let keypair = KeyPair::new();
    let public_key = hex::encode(keypair.public_key);
    let secret_key = hex::encode(&keypair.secret_key[..]);

    let label = match label {
        Some(l) => l,
        None => Input::new()
            .with_prompt("Enter a label for the API key")
            .interact_text()?,
    };

    let expiry = match expiry {
        Some(e) => e,
        None => Input::new()
            .with_prompt("Enter an expiry date for the API key (ISO 8601 format)")
            .default("2099-12-31T23:59:59Z".to_string())
            .interact_text()?,
    };

    let mut authorized_keys = auth::keys::AuthorizedKeys::load_from_file("authorized_keys.json")
        .unwrap_or_else(|_| auth::keys::AuthorizedKeys::new());

    if authorized_keys.is_key_authorized(&public_key) {
        println!("Key already exists in authorized_keys file.");
    } else {
        let confirm = Confirm::new()
            .with_prompt("Do you want to add this key to the authorized_keys file?")
            .interact()?;

        if confirm {
            authorized_keys.add_key(label.clone(), public_key.clone(), expiry.clone());
            authorized_keys.save_to_file("authorized_keys.json")?;
            println!("Key added to authorized_keys file.");
        } else {
            println!("Key not added to authorized_keys file.");
        }
    }

    println!("Public Key: {}", public_key);
    println!("Secret Key: {}", secret_key);
    println!("Label: {}", label);
    println!("Expiry: {}", expiry);
    Ok(())
}
