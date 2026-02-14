use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum TransactionType {
    Init { data: Value },
    ReleaseSet { id: String, data: Value },
    TransferCard { card_id: String, sender: String, receiver: String },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockTransaction {
    pub transaction_type: TransactionType,
}

impl BlockTransaction {
    pub fn new(transaction_type: TransactionType) -> Self {
        BlockTransaction { transaction_type }
    }
}
