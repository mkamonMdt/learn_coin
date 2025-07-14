use crate::primitives::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: String,
    pub state_root: String,
    pub total_fees: f64,
}

impl Block {
    pub fn new(
        transactions: Vec<Transaction>,
        previous_hash: String,
        validator: String,
        state_root: String,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let total_fees = transactions.iter().map(|tx| tx.fee).sum();
        let mut block = Block {
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            validator,
            state_root,
            total_fees,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!(
            "{}{}{}{}{}{}",
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap(),
            self.previous_hash,
            self.validator,
            self.state_root,
            self.total_fees,
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
