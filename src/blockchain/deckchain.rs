use std::fs::read_to_string;

use sha3::{Digest, Sha3_256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::blockchain::block::Block;
use crate::blockchain::chain::BlockChain;
use crate::blockchain::transaction::{BlockTransaction, TransactionType};
use crate::card::seriesreleasestate::TradingCardSeriesReleaseState;
use crate::config::Config;
use crate::error::{DeckForgeError, Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct DeckChain {
    pub data_dir: String,
    pub blockchain: BlockChain,
    pub series_states: Vec<TradingCardSeriesReleaseState>,
}

impl DeckChain {
    const BLOCKCHAIN_FILENAME: &'static str = "blockchain.json";

    pub fn new(config_path: &str) -> Result<Self> {
        let config = Config::load(config_path)?;
        let blockchain_data_dir = &config.data_dir;
        let blockchain = DeckChain::get_init_blockchain(blockchain_data_dir)?;
        let mut deckchain = DeckChain {
            data_dir: blockchain_data_dir.to_string(),
            blockchain,
            series_states: Vec::new(),
        };

        let releases = deckchain.card_series_releases();

        for release in releases {
            if let Some(id) = release.get("id").and_then(|v| v.as_str()) {
                let series_state =
                    TradingCardSeriesReleaseState::from_deckchain(&deckchain, id.to_string())?;
                deckchain.series_states.push(series_state);
            }
        }

        Ok(deckchain)
    }

    fn get_init_blockchain(blockchain_data_dir: &str) -> Result<BlockChain> {
        let path = format!("{}/{}", blockchain_data_dir, DeckChain::BLOCKCHAIN_FILENAME);
        BlockChain::new(&path, Value::Null)
    }

    pub fn save(&self) -> Result<()> {
        let full_path = format!("{}/{}", self.data_dir, DeckChain::BLOCKCHAIN_FILENAME);
        self.blockchain.save(&full_path)
    }

    pub fn get_blocks(&self) -> &[Block] {
        self.blockchain.get_blocks()
    }

    pub fn add_block(&mut self, transactions: Vec<BlockTransaction>) -> Result<()> {
        self.blockchain.add_block(transactions)
    }

    pub fn init_data(&self) -> Result<Value> {
        self.blockchain.get_init_data()
    }

    /// Retrieves the data from all card series releases stored in the blockchain.
    pub fn card_series_releases(&self) -> Vec<Value> {
        self.blockchain
            .blocks
            .iter()
            .filter_map(|block| {
                block.transactions.iter().find_map(|tx| match &tx.transaction_type {
                    TransactionType::ReleaseSet { data, .. } => Some(data.clone()),
                    _ => None,
                })
            })
            .collect()
    }

    /// Retrieves the data from a specific card series release stored in the blockchain.
    pub fn card_series_release(&self, series_id: &str) -> Result<Value> {
        let all_releases = self.card_series_releases();

        if all_releases.is_empty() {
            return Err(DeckForgeError::NoReleasesFound);
        }

        all_releases
            .into_iter()
            .find(|release| {
                release.get("id").and_then(|v| v.as_str()) == Some(series_id)
            })
            .ok_or_else(|| DeckForgeError::SeriesNotFound {
                id: series_id.to_string(),
            })
    }

    /// Creates a new card series release in the blockchain.
    pub fn do_release_series(&mut self, series_file: String) -> Result<()> {
        let series_data = read_to_string(&series_file)?;
        let series_json: Value = serde_json::from_str(&series_data)?;

        self.validate_series(&series_json)?;

        let mut hasher = Sha3_256::new();
        hasher.update(series_data.as_bytes());
        let series_hash = format!("{:x}", hasher.finalize());

        let transaction = BlockTransaction::new(TransactionType::ReleaseSet {
            id: series_hash,
            data: series_json,
        });

        self.blockchain.add_block(vec![transaction])?;
        self.save()?;
        println!("ReleaseSet transaction inserted successfully.");
        Ok(())
    }

    /// Validates configuration data of a card series.
    pub fn validate_series(&self, series_json: &Value) -> Result<()> {
        let series_id = series_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeckForgeError::Validation {
                reason: "Series JSON does not contain an 'id' field".to_string(),
            })?;

        if self.card_series_release(series_id).is_ok() {
            return Err(DeckForgeError::AlreadyReleased {
                id: series_id.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::testing::init_test_data_dir;

    #[test]
    fn test_release_series_not_found() {
        let testing_path = init_test_data_dir();
        let deckchain = DeckChain::new(&testing_path).unwrap();
        let series_data = deckchain.card_series_release("LEGACYDECK-1");
        assert!(series_data.is_err());
    }

    #[test]
    fn test_release_series() {
        let testing_path = init_test_data_dir();
        let mut deckchain = DeckChain::new(&testing_path).unwrap();
        let series_file = "test/series.json".to_string();
        let release_result = deckchain.do_release_series(series_file);
        assert!(release_result.is_ok());

        let series_data = deckchain.card_series_release("LEGACYDECK-1").unwrap();
        assert_eq!(series_data.get("id").unwrap().as_str().unwrap(), "LEGACYDECK-1");
    }

    #[test]
    fn test_release_series_twice() {
        let testing_path = init_test_data_dir();
        let mut deckchain = DeckChain::new(&testing_path).unwrap();
        let series_file = "test/series.json".to_string();

        let release_result = deckchain.do_release_series(series_file.clone());
        assert!(release_result.is_ok());

        let series_data = deckchain.card_series_release("LEGACYDECK-1").unwrap();
        assert_eq!(series_data.get("id").unwrap().as_str().unwrap(), "LEGACYDECK-1");
        let release_result = deckchain.do_release_series(series_file);
        assert!(release_result.is_err());

        let series_data = deckchain.card_series_releases();
        assert_eq!(series_data.len(), 1);
    }
}
