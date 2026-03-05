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
use crate::config::Config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;

    match cli.command {
        Commands::GenerateKey { label, expiry } => {
            commands::keys::generate_key(label, expiry, &config)?;
        }

        Commands::StartServer => {
            tracing::info!("Starting server...");
            server::start_server(config).await?;
        }

        Commands::InsertReleaseSet { series_file } => {
            let mut deckchain = DeckChain::new(&config)?;
            if let Err(e) = deckchain.do_release_series(series_file) {
                tracing::error!("Error: {}", e);
            }
        }
    }

    Ok(())
}
