use crate::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: String,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, previous_hash: String, validator: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            validator,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!(
            "{}{}{}{}",
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap(),
            self.previous_hash,
            self.validator
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
