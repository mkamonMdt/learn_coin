use std::{collections::HashMap, thread::current, u64};

use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{digest::block_buffer, Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    timestamp: i64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    validator: String,
}

impl Block {
    fn new(transactions: Vec<Transaction>, previous_hash: String, validator: String) -> Self {
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

    fn calculate_hash(&self) -> String {
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

#[derive(Debug)]
struct Wallet {
    balance: f64,
    staked: f64,
}

#[derive(Debug)]
struct Blockchain {
    chain: Vec<Block>,
    wallets: HashMap<String, Wallet>,
    slots_per_epoch: usize,
}

impl Blockchain {
    fn new() -> Self {
        let mut wallets = HashMap::new();
        wallets.insert(
            "Genesis".to_string(),
            Wallet {
                balance: 1000.0,
                staked: 0.0,
            },
        );
        let genesis_block = Block::new(
            vec![Transaction {
                sender: "Genesis".to_string(),
                receiver: "System".to_string(),
                amount: 1000.0,
            }],
            "0".to_string(),
            "Genesis".to_string(),
        );
        Blockchain {
            chain: vec![genesis_block],
            wallets,
            slots_per_epoch: 10,
        }
    }

    fn stake(&mut self, user: &str, amount: f64) -> Result<(), String> {
        let wallet = self.wallets.get_mut(user).ok_or("User not founf")?;
        if wallet.balance >= amount {
            wallet.balance -= amount;
            wallet.staked += amount;
            Ok(())
        } else {
            Err("Insufficient balance to stake".to_string())
        }
    }

    fn get_epoch(&self, block_height: usize) -> usize {
        block_height / self.slots_per_epoch
    }

    fn get_epoch_seed(&self, epoch: usize) -> String {
        if epoch == 0 {
            return "0".to_string();
        }

        let last_block_of_prev_epoch = epoch * self.slots_per_epoch - 1;
        assert!(
            last_block_of_prev_epoch < self.chain.len(),
            "Chain of len={} too short for epoch={}",
            last_block_of_prev_epoch,
            epoch
        );
        self.chain[last_block_of_prev_epoch].hash.clone()
    }

    fn assign_slots(&self, epoch: usize) -> Vec<String> {
        let total_stake: f64 = self.wallets.values().map(|w| w.staked).sum();
        if total_stake == 0.0 {
            return vec!["Genesis".to_string(); self.slots_per_epoch];
        }

        let seed = self.get_epoch_seed(epoch);
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        let result = hasher.finalize();
        let seed_value = u64::from_le_bytes(result[..8].try_into().unwrap());

        let mut slots = Vec::with_capacity(self.slots_per_epoch);
        //        let mut remaining_slots = self.slots_per_epoch;
        let mut stake_pool: Vec<(&String, f64)> =
            self.wallets.iter().map(|(k, v)| (k, v.staked)).collect();

        for i in 0..self.slots_per_epoch {
            if stake_pool.is_empty() {
                slots.push("Genesis".to_string());
                continue;
            }

            let total_remaining_stake: f64 = stake_pool.iter().map(|(_, s)| s).sum();
            let slot_seed = seed_value.wrapping_add(i as u64);
            let random_point = slot_seed as f64 % total_remaining_stake;
            let mut cumulative = 0.0;

            for (j, (user, stake)) in stake_pool.iter().enumerate() {
                cumulative += stake;
                if cumulative >= random_point {
                    slots.push(user.to_string());
                    stake_pool.remove(j);
                    break;
                }
            }
            if slots.len() <= i {
                slots.push(stake_pool[0].0.to_string());
            }
        }
        slots
    }

    fn select_validator(&self, block_height: usize) -> Option<String> {
        let epoch = self.get_epoch(block_height);
        let slot_in_epoch = block_height % self.slots_per_epoch;
        let validator_to_slot_assignment = self.assign_slots(epoch);

        Some(validator_to_slot_assignment[slot_in_epoch].clone())
    }

    fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), String> {
        let block_height = self.chain.len();
        let validator = self
            .select_validator(block_height)
            .ok_or("No validators available")?;
        let previous_block = self.chain.last().unwrap();
        let new_block = Block::new(
            transactions.clone(),
            previous_block.hash.clone(),
            validator.clone(),
        );

        for tx in &transactions {
            let sender_wallet = self.wallets.get_mut(&tx.sender).ok_or("Sender not found")?;
            if sender_wallet.balance < tx.amount {
                return Err("Insufficient balance".to_string());
            }
            sender_wallet.balance -= tx.amount;

            let receiver_wallet = self.wallets.entry(tx.receiver.clone()).or_insert(Wallet {
                balance: 0.0,
                staked: 0.0,
            });
            receiver_wallet.balance += tx.amount;
        }

        let validator_wallet = self.wallets.get_mut(&validator).unwrap();
        validator_wallet.balance += 10.0;

        self.chain.push(new_block);
        Ok(())
    }

    fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.calculate_hash() || current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        let first_block = blockchain.chain.first().unwrap();
        assert_eq!(first_block.previous_hash, "0".to_owned());
        assert_eq!(first_block.validator, "Genesis".to_owned());
    }
}
