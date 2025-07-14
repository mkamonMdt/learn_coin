use serde::Serialize;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Serialize)]
pub struct PendingUnstake {
    pub amount: f64,
    pub effective_epoch: usize,
}

#[derive(Debug, Serialize)]
pub struct Wallet {
    pub balance: f64,
    pub staked: f64,
    pub pending_unstakes: VecDeque<PendingUnstake>,
}

impl Wallet {
    pub fn new(balance: f64) -> Self {
        Self {
            balance,
            staked: 0.0,
            pending_unstakes: VecDeque::new(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Wallets {
    pub wallets: HashMap<String, Wallet>,
}
