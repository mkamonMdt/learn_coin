use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    Transfer {
        sender: String,
        receiver: String,
        amount: f64,
    },
    Stake {
        user: String,
        amount: f64,
    },
    Unstake {
        user: String,
        amount: f64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub tx_type: TransactionType,
}
