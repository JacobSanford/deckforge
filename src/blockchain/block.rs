use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use serde_json::Value;
use sha3::{Digest, Sha3_256};

use crate::blockchain::chain::BlockChain;
use crate::blockchain::transaction::{BlockTransaction, TransactionType};

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub transactions: Vec<BlockTransaction>,
    pub hash: String,
}

impl Block {
    pub fn new(chain: BlockChain, transactions: Vec<BlockTransaction>) -> Self {
        let previous_block = chain.blocks.last().unwrap();
        let index = previous_block.index + 1;
        let previous_hash = previous_block.hash();

        let mut block = Block {
            index,
            previous_hash,
            timestamp: Block::cur_microtime(),
            transactions,
            hash: String::new(),
        };
        block.hash = block.hash();
        block
    }

    pub fn new_genesis(init_data: Value) -> Self {
        let transactions = vec![BlockTransaction::new(
            TransactionType::Init { data: init_data }
        )];
        let mut block = Block {
            index: 0,
            previous_hash: String::from("0"),
            timestamp: Block::cur_microtime(),
            transactions,
            hash: String::new(),
        };
        block.hash = block.hash();
        block
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha3_256::new();
        let data = format!(
            "{}{}{}{:?}",
            self.index,
            self.previous_hash,
            self.timestamp,
            self.transactions
        );
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn cur_microtime() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros()
    }
}