use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::blockchain::transaction::{BlockTransaction, TransactionType};
use crate::blockchain::block::Block;

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl BlockChain {
    pub const NULL_ADDRESS: &'static str = "0x0000000000000000000000000000000000000000";

    pub fn new(storage_path: &str, init_data: Value) -> io::Result<Self> {
        match BlockChain::load(storage_path) {
            Ok(blockchain) => Ok(blockchain),
            Err(_) => BlockChain::init(storage_path, init_data),
        }
    }

    pub fn load(storage_path: &str) -> io::Result<Self> {
        if Path::new(&storage_path).exists() {
            let mut file = File::open(&storage_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let blockchain: BlockChain = serde_json::from_str(&contents)?;
            Ok(blockchain)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Blockchain file not found"))
        }
    }

    pub fn init(storage_path: &str, init_data: Value) -> io::Result<Self> {
        let genesis_block = Block::new_genesis(init_data);

        let blockchain = BlockChain {
            blocks: vec![genesis_block],
        };

        let serialized_blockchain = serde_json::to_string(&blockchain)?;
        let mut file = File::create(storage_path)?;
        file.write_all(serialized_blockchain.as_bytes())?;

        Ok(blockchain)
    }

    pub fn save(&self, storage_path: &str) -> io::Result<()> {
        let contents = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(&storage_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn get_init_data(&self) -> Value {
        let init_transaction = self.blocks.get(0).unwrap().transactions.get(0).unwrap();
        match &init_transaction.transaction_type {
            TransactionType::Init { data } => data.clone(),
            _ => Value::Null,
        }
    }

    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.blocks.get(index as usize)
    }
    
    pub fn get_blocks(&self) -> &Vec<Block> {
        &self.blocks
    }

    pub fn add_block(&mut self, transactions: Vec<BlockTransaction>) {
        let block = Block::new(self.clone(), transactions);
        self.blocks.push(block);
    }

    pub fn get_blocks_by_transaction_type(&self, transaction_type: TransactionType) -> Vec<&Block> {
        self.blocks.iter().filter(|block| {
            block.transactions.iter().any(|transaction| {
                transaction.transaction_type == transaction_type
            })
        }).collect()
    }

}