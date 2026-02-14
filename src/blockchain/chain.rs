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
            Ok(blockchain)
        } else {
            Err(DeckForgeError::BlockchainNotFound {
                path: storage_path.to_string(),
            })
        }
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

    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.blocks.get(index as usize)
    }

    pub fn get_blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn add_block(&mut self, transactions: Vec<BlockTransaction>) -> Result<()> {
        let previous = self.blocks.last().ok_or(DeckForgeError::EmptyChain)?;
        let block = Block::new(previous, transactions);
        self.blocks.push(block);
        Ok(())
    }

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
