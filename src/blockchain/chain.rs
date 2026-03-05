use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::blockchain::block::Block;
use crate::blockchain::transaction::{BlockTransaction, TransactionType};
use crate::error::{DeckForgeError, Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl BlockChain {
    #[allow(dead_code)] // blockchain constant for transaction addresses
    pub const NULL_ADDRESS: &'static str = "0x0000000000000000000000000000000000000000";

    pub fn new(storage_path: &str, init_data: Value) -> Result<Self> {
        match BlockChain::load(storage_path) {
            Ok(blockchain) => Ok(blockchain),
            Err(_) => BlockChain::init(storage_path, init_data),
        }
    }

    pub fn load(storage_path: &str) -> Result<Self> {
        if Path::new(storage_path).exists() {
            let contents = fs::read_to_string(storage_path)?;
            let blockchain: BlockChain = serde_json::from_str(&contents)?;
            blockchain.validate()?;
            Ok(blockchain)
        } else {
            Err(DeckForgeError::BlockchainNotFound {
                path: storage_path.to_string(),
            })
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.blocks.is_empty() {
            return Err(DeckForgeError::EmptyChain);
        }

        let genesis = &self.blocks[0];
        if genesis.index != 0 {
            return Err(DeckForgeError::Validation {
                reason: format!("Genesis block has index {}, expected 0", genesis.index),
            });
        }
        if genesis.previous_hash != "0" {
            return Err(DeckForgeError::Validation {
                reason: "Genesis block has invalid previous_hash".to_string(),
            });
        }

        for (i, block) in self.blocks.iter().enumerate() {
            let recomputed = block.hash();
            if block.hash != recomputed {
                return Err(DeckForgeError::Validation {
                    reason: format!(
                        "Block {} hash mismatch: stored={}, computed={}",
                        block.index, block.hash, recomputed
                    ),
                });
            }

            if block.index != i as u64 {
                return Err(DeckForgeError::Validation {
                    reason: format!(
                        "Block index {} at position {}", block.index, i
                    ),
                });
            }

            if i > 0 {
                let prev = &self.blocks[i - 1];
                if block.previous_hash != prev.hash {
                    return Err(DeckForgeError::Validation {
                        reason: format!(
                            "Block {} previous_hash doesn't match block {} hash",
                            block.index,
                            prev.index
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn init(storage_path: &str, init_data: Value) -> Result<Self> {
        let genesis_block = Block::new_genesis(init_data);

        let blockchain = BlockChain {
            blocks: vec![genesis_block],
        };

        let serialized = serde_json::to_string(&blockchain)?;
        fs::write(storage_path, serialized)?;

        Ok(blockchain)
    }

    pub fn save(&self, storage_path: &str) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(storage_path, contents)?;
        Ok(())
    }

    #[allow(dead_code)] // public API
    pub fn get_init_data(&self) -> Result<Value> {
        let genesis = self.blocks.first().ok_or(DeckForgeError::EmptyChain)?;
        let init_tx = genesis
            .transactions
            .first()
            .ok_or(DeckForgeError::EmptyChain)?;
        match &init_tx.transaction_type {
            TransactionType::Init { data } => Ok(data.clone()),
            _ => Ok(Value::Null),
        }
    }

    #[allow(dead_code)] // public API
    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.blocks.get(index as usize)
    }

    #[allow(dead_code)] // public API
    pub fn get_blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn add_block(&mut self, transactions: Vec<BlockTransaction>) -> Result<()> {
        let previous = self.blocks.last().ok_or(DeckForgeError::EmptyChain)?;
        let block = Block::new(previous, transactions);
        self.blocks.push(block);
        Ok(())
    }

    #[allow(dead_code)] // public API
    pub fn get_blocks_by_transaction_type(&self, transaction_type: TransactionType) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|block| {
                block
                    .transactions
                    .iter()
                    .any(|tx| tx.transaction_type == transaction_type)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_chain() -> BlockChain {
        let genesis = Block::new_genesis(Value::Null);
        let block1 = Block::new(&genesis, vec![]);
        BlockChain {
            blocks: vec![genesis, block1],
        }
    }

    #[test]
    fn test_validate_valid_chain() {
        let chain = test_chain();
        assert!(chain.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_chain() {
        let chain = BlockChain { blocks: vec![] };
        assert!(chain.validate().is_err());
    }

    #[test]
    fn test_validate_tampered_hash() {
        let mut chain = test_chain();
        chain.blocks[1].hash = "tampered".to_string();
        let err = chain.validate().unwrap_err().to_string();
        assert!(err.contains("hash mismatch"));
    }

    #[test]
    fn test_validate_broken_linkage() {
        let mut chain = test_chain();
        chain.blocks[1].previous_hash = "wrong".to_string();
        // Recompute hash so the hash itself is valid for its data
        chain.blocks[1].hash = chain.blocks[1].hash();
        let err = chain.validate().unwrap_err().to_string();
        assert!(err.contains("previous_hash doesn't match"));
    }
}
