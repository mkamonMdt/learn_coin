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
    DeployContract {
        code: Vec<u8>,
    },
    CallContract {
        contract_address: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub tx_type: TransactionType,
    pub fee: f64,
}

impl Transaction {
    pub fn new(sender: String, tx_type: TransactionType, fee: f64) -> Self {
        Transaction {
            sender,
            tx_type,
            fee,
        }
    }
}
