mod api;
mod auth;
mod blockchain;
mod card;
mod commands;
mod config;
mod crypto;
mod error;

use clap::Parser;

use crate::api::server;
use crate::commands::commands::Commands;
use crate::blockchain::deckchain::DeckChain;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config_path = "config.toml".to_string();

    match cli.command {
        Commands::GenerateKey { label, expiry } => {
            commands::keys::generate_key(label, expiry)?;
        }

        Commands::StartServer => {
            println!("Starting server...");
            server::start_server(config_path).await?;
        }

        Commands::InsertReleaseSet { series_file } => {
            let mut deckchain = DeckChain::new(&config_path)?;
            if let Err(e) = deckchain.do_release_series(series_file) {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
