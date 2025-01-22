mod api;
mod auth;
mod card;
mod config;
mod blockchain;
mod commands;
mod crypto;
mod logging;

use clap::Parser;
use env_logger;

use crate::commands::commands::Commands;
use crate::config::Config;
use crate::api::server;

use blockchain::deckchain::{self, DeckChain};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let config_path = "config.toml".to_string();
    
    match cli.command {
        // Key Generation.
        Commands::GenerateKey { label, expiry } => {
            commands::keys::generate_key(label, expiry).await;
        }

        // Start the server.
        Commands::StartServer => {
            println!("Starting server...");
            server::start_server(config_path).await.expect("Failed to start server");
        }

        // Insert ReleaseSet transaction.
        Commands::InsertReleaseSet { series_file } => {
            let config_path = "config.toml";
            let mut deckchain = DeckChain::new(config_path).await.unwrap();
            if let Err(e) = deckchain.do_release_series(series_file).await {
                eprintln!("Error: {}", e);
            }
        }
    }
}
