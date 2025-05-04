use std::fs::{read_to_string, File};
use std::io::{self, Read, Write};
use std::error::Error;

use sha3::{Digest, Sha3_256};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::blockchain::transaction::{BlockTransaction, TransactionType};
use crate::blockchain::block::Block;
use crate::blockchain::chain::BlockChain;

use crate::card::seriesreleasestate::TradingCardSeriesReleaseState;

#[derive(Clone, Serialize, Deserialize)]
pub struct DeckChain {
    pub data_dir: String,
    pub blockchain: BlockChain,
    pub series_states: Vec<TradingCardSeriesReleaseState>,
}

impl DeckChain {
    const BLOCKCHAIN_FILENAME: &'static str = "blockchain.json";

    pub async fn new(blockchain_data_dir: &str) -> io::Result<Self> {
        let blockchain = DeckChain::get_init_blockchain(blockchain_data_dir);
        let series_states = Vec::new();

        let mut deckchain = DeckChain {
            data_dir: blockchain_data_dir.to_string(),
            blockchain: blockchain.clone(),
            series_states: series_states.clone(),
        };

        let releases = deckchain.card_series_releases().await.unwrap();

        for release in releases {
            let series_id = release.get("id").unwrap().as_str().unwrap();
            let series_state = TradingCardSeriesReleaseState::from_deckchain(deckchain.clone(), series_id.to_string()).await;
            deckchain.series_states.push(series_state);
        }

        Ok(deckchain)
    }

    fn get_init_blockchain(blockchain_data_dir: &str) -> BlockChain {
        let blockchain_storage_path = format!("{}/{}", blockchain_data_dir, DeckChain::BLOCKCHAIN_FILENAME);
        BlockChain::new(&blockchain_storage_path, Value::Null).unwrap()
    }

    pub fn save(&self) -> io::Result<()> {
        let full_path = format!("{}/{}", self.data_dir, DeckChain::BLOCKCHAIN_FILENAME);
        self.blockchain.save(&full_path)
    }

    pub fn get_blocks(&self) -> &Vec<Block> {
        self.blockchain.get_blocks()
    }

    pub fn add_block(&mut self, transactions: Vec<BlockTransaction>) {
        self.blockchain.add_block(transactions);
    }

    pub fn init_data(&self) -> Value {
        self.blockchain.get_init_data()
    }

    /// Retrieves the data from all card series releases stored in the blockchain.
    ///
    /// # Returns
    /// 
    /// * `Result<Vec<Value>, Box<dyn Error>>` - A result containing a vector of `Value`
    ///   representing the release data, or an error if the operation fails.
    pub async fn card_series_releases(&self) -> Result<Vec<Value>, Box<dyn Error>> {
        let series_release_blocks = self.blockchain.blocks.iter().filter_map(|block| {
            block.transactions.iter().find_map(|transaction| {
                match &transaction.transaction_type {
                    TransactionType::ReleaseSet { id, data } => {
                        Some(data)
                    },
                    _ => None,
                }
            })
        });

        let series_data: Vec<Value> = series_release_blocks.cloned().collect();
        Ok(series_data)
    }

    /// Retrieves the data from a specific card series release stored in the blockchain.
    ///
    /// # Arguments
    ///
    /// * `series_id` - The ID of the series to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Value, Box<dyn Error>>` - A result containing the release data, or an error if the
    ///   operation fails.
    pub async fn card_series_release(&self, series_id: String) -> Result<Value, Box<dyn Error>> {
        let all_releases = self.card_series_releases().await?;

        if all_releases.is_empty() {
            return Err("No series releases found".into());
        }

        let series_data = all_releases.iter().find(|release| {
            release.get("id").unwrap().as_str().unwrap() == series_id
        });

        match series_data {
            Some(data) => Ok(data.clone()),
            None => Err("Series not found".into()),
        }
    }

    /// Creates a new card series release in the blockchain.
    ///
    /// # Arguments
    ///
    /// * `series_file` - The path to the file containing the series data.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - A result indicating success or failure.
    pub async fn do_release_series(&mut self, series_file: String) -> Result<(), Box<dyn Error>> {
        let series_data = read_to_string(series_file).expect("Failed to read series file");

        let series_json: Value = serde_json::from_str(&series_data).expect("Failed to parse series file");
        if let Err(e) = self.validate_series(&series_json).await {
            return Err(e);
        }

        let mut hasher = Sha3_256::new();
        hasher.update(series_data.as_bytes());
        let series_hash = format!("{:x}", hasher.finalize());

        let transaction = BlockTransaction::new(TransactionType::ReleaseSet {
            id: series_hash,
            data: series_json,
        });

        self.blockchain.add_block(vec![transaction]);

        self.save().expect("Failed to save blockchain");
        println!("ReleaseSet transaction inserted successfully.");
        Ok(())
    }

    /// Validates configuration data of a card series.
    ///
    /// This is likely will be a monstrosity of a function.
    ///
    /// # Arguments
    ///
    /// * `series_json` - The JSON data representing the card series.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - A result indicating success or failure.
    pub async fn validate_series(&self, series_json: &Value) -> Result<(), Box<dyn Error>> {
        let series_id = series_json.get("id").unwrap().as_str().unwrap();
        let release = self.card_series_release(series_id.to_string()).await;
        if release.is_ok() {
            return Err("Series has already been released".into());
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::blockchain::testing::init_test_data_dir;

    #[tokio::test]
    async fn test_release_series_not_found() {
        let testing_path = init_test_data_dir();
        let deckchain = DeckChain::new(&testing_path).await.unwrap();
        let series_data = deckchain.card_series_release("LEGACYDECK-1".to_string()).await;
        assert!(series_data.is_err());
    }

    #[tokio::test]
    async fn test_release_series() {
        let testing_path = init_test_data_dir();
        let mut deckchain = DeckChain::new(&testing_path).await.unwrap();
        let series_file = "test/series.json".to_string();
        let release_result = deckchain.do_release_series(series_file).await;
        assert!(release_result.is_ok());

        let series_data = deckchain.card_series_release("LEGACYDECK-1".to_string()).await.unwrap();
        assert_eq!(series_data.get("id").unwrap().as_str().unwrap(), "LEGACYDECK-1");
    }

    #[tokio::test]
    async fn test_release_series_twice() {
        let testing_path = init_test_data_dir();
        let mut deckchain = DeckChain::new(&testing_path).await.unwrap();
        let series_file = "test/series.json".to_string();

        let release_result = deckchain.do_release_series(series_file.clone()).await;
        assert!(release_result.is_ok());

        let series_data = deckchain.card_series_release("LEGACYDECK-1".to_string()).await.unwrap();
        assert_eq!(series_data.get("id").unwrap().as_str().unwrap(), "LEGACYDECK-1");
        let release_result = deckchain.do_release_series(series_file).await;
        assert!(release_result.is_err());
    
        let series_data = deckchain.card_series_releases().await.unwrap();
        assert_eq!(series_data.len(), 1);
    }

}
