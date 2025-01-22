use std::fs;

use clap::Subcommand;
use serde_json::Value;
use sha3::{Digest, Sha3_256};

use crate::blockchain::chain::BlockChain;
use crate::config::Config;
use crate::blockchain::transaction::{BlockTransaction, TransactionType};

#[derive(Subcommand)]
pub enum Commands {
    GenerateKey {
        #[arg(short, long)]
        label: Option<String>,

        #[arg(short, long)]
        expiry: Option<String>,
    },
    StartServer,
    InsertReleaseSet {
        #[arg(short, long)]
        series_file: String,
    },
}

impl Commands {

}
