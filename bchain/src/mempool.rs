use crate::transaction::{self, Transaction};

struct Mempool {
    transactions: Vec<Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Mempool {
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn get_transactions(&mut self) -> Vec<Transaction> {
        self.transactions.drain(..).collect()
    }
}
