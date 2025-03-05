use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TransactionType {
    Transfer {
        sender: String,
        receiver: String,
        amount: f64,
    },
    Stake {
        user: String,
        amount: f64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    tx_type: TransactionType,
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
        wallets.insert("Genesis".to_string(), Wallet { balance: 1000.0 });
        let genesis_block = Block::new(
            vec![Transaction {
                tx_type: TransactionType::Transfer {
                    sender: "Genesis".to_string(),
                    receiver: "System".to_string(),
                    amount: 1000.0,
                },
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

    fn get_stake_pool(&self, epoch: usize) -> HashMap<String, f64> {
        let up_to_block = self.get_validators_consensus_block(epoch);
        let mut stake_pool = HashMap::new();
        let up_to_block = up_to_block.min(self.chain.len() - 1);
        for block in &self.chain[..=up_to_block] {
            for tx in &block.transactions {
                if let TransactionType::Stake { user, amount } = &tx.tx_type {
                    *stake_pool.entry(user.clone()).or_insert(0.0) += amount;
                }
            }
        }
        stake_pool
    }

    fn get_epoch(&self, block_height: usize) -> usize {
        block_height / self.slots_per_epoch
    }

    fn get_validators_consensus_block(&self, epoch: usize) -> usize {
        if epoch == 0 {
            0
        } else {
            epoch * self.slots_per_epoch - 1
        }
    }

    fn get_epoch_seed(&self, epoch: usize) -> String {
        if epoch == 0 {
            return "0".to_string();
        }

        let validators_consensus_block = self.get_validators_consensus_block(epoch);
        assert!(
            validators_consensus_block < self.chain.len(),
            "Chain of len={} too short for epoch={}",
            validators_consensus_block,
            epoch
        );
        self.chain[validators_consensus_block].hash.clone()
    }

    fn get_validator_for_slots(&self, epoch: usize, slot: usize) -> Option<String> {
        let stake_pool = self.get_stake_pool(epoch);
        let total_stake: f64 = stake_pool.values().sum();
        if total_stake == 0.0 {
            return Some("Genesis".to_string());
        }

        let seed = self.get_epoch_seed(epoch);
        let slot_seed = format!("{}{}", seed, slot);
        let mut hasher = Sha256::new();
        hasher.update(&slot_seed);
        let result = hasher.finalize();
        let seed_value = u64::from_le_bytes(result[..8].try_into().unwrap());
        let random_point = seed_value as f64 % total_stake;

        let mut cumulative = 0.0;
        for (user, stake) in &stake_pool {
            cumulative += stake;
            if cumulative >= random_point {
                return Some(user.clone());
            }
        }
        None
    }

    fn select_validator(&self) -> Option<String> {
        let block_height = self.chain.len();
        let epoch = self.get_epoch(block_height);
        let slot_in_epoch = block_height % self.slots_per_epoch;

        self.get_validator_for_slots(epoch, slot_in_epoch)
    }

    fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), String> {
        let validator = self.select_validator().ok_or("No validators available")?;
        let previous_block = self.chain.last().unwrap();
        let new_block = Block::new(
            transactions.clone(),
            previous_block.hash.clone(),
            validator.clone(),
        );

        for tx in &transactions {
            match &tx.tx_type {
                TransactionType::Stake { user, amount } => {
                    let wallet = self.wallets.get_mut(user).ok_or("User not found")?;
                    if wallet.balance < *amount {
                        return Err("Insufficient ballance to stake".to_string());
                    }
                    wallet.balance -= *amount;
                }
                TransactionType::Transfer {
                    sender,
                    receiver,
                    amount,
                } => {
                    let sender_wallet = self.wallets.get_mut(sender).ok_or("Sender not found")?;
                    if sender_wallet.balance < *amount {
                        return Err("Insufficient balance".to_string());
                    }
                    sender_wallet.balance -= *amount;

                    let receiver_wallet = self
                        .wallets
                        .entry(receiver.clone())
                        .or_insert(Wallet { balance: 0.0 });
                    receiver_wallet.balance += *amount;
                }
            }
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
    const INITIAL_AMOUNT: f64 = 100.0;
    const EXCEESIVE_AMOUNT: f64 = 150.0;
    const SUFFICIENT_AMOUNT: f64 = 60.0;

    fn initiate_account(blockchain: &mut Blockchain, user: String) {
        blockchain
            .add_block(vec![Transaction {
                tx_type: TransactionType::Transfer {
                    sender: "Genesis".to_string(),
                    receiver: user,
                    amount: INITIAL_AMOUNT,
                },
            }])
            .unwrap();
    }

    fn put_stake(blockchain: &mut Blockchain, user: String, amount: f64) -> Result<(), String> {
        blockchain.add_block(vec![Transaction {
            tx_type: TransactionType::Stake { user, amount },
        }])
    }

    fn transfer(
        blockchain: &mut Blockchain,
        sender: String,
        receiver: String,
        amount: f64,
    ) -> Result<(), String> {
        blockchain.add_block(vec![Transaction {
            tx_type: TransactionType::Transfer {
                sender,
                receiver,
                amount,
            },
        }])
    }

    #[test]
    fn test_genesis_block() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        let first_block = blockchain.chain.first().unwrap();
        assert_eq!(first_block.previous_hash, "0".to_owned());
        assert_eq!(first_block.validator, "Genesis".to_owned());
    }

    #[test]
    fn test_ok_when_put_valid_stake() {
        let mut blockchain = Blockchain::new();
        println!("Genesis block: {:?}", blockchain.chain[0]);

        let account_1 = "Allice".to_string();
        initiate_account(&mut blockchain, account_1.clone());
        assert!(put_stake(&mut blockchain, account_1, SUFFICIENT_AMOUNT).is_ok());
    }

    #[test]
    fn test_error_when_put_too_high_stake() {
        let mut blockchain = Blockchain::new();
        println!("Genesis block: {:?}", blockchain.chain[0]);

        let account_1 = "Allice".to_string();
        initiate_account(&mut blockchain, account_1.clone());
        assert!(put_stake(&mut blockchain, account_1, EXCEESIVE_AMOUNT).is_err());
    }

    #[test]
    fn test_error_when_too_high_stake_put_after_tx() {
        let mut blockchain = Blockchain::new();
        println!("Genesis block: {:?}", blockchain.chain[0]);

        let account_1 = "Allice".to_string();
        let account_2 = "Bob".to_string();
        initiate_account(&mut blockchain, account_1.clone());
        initiate_account(&mut blockchain, account_2.clone());

        assert!(transfer(
            &mut blockchain,
            account_1.clone(),
            account_2,
            SUFFICIENT_AMOUNT
        )
        .is_ok());
        assert!(put_stake(&mut blockchain, account_1, SUFFICIENT_AMOUNT).is_err());
    }

    #[test]
    fn test_error_when_too_high_tx_after_stake_put() {
        let mut blockchain = Blockchain::new();
        println!("Genesis block: {:?}", blockchain.chain[0]);

        let account_1 = "Allice".to_string();
        let account_2 = "Bob".to_string();
        initiate_account(&mut blockchain, account_1.clone());
        initiate_account(&mut blockchain, account_2.clone());

        assert!(put_stake(&mut blockchain, account_1.clone(), SUFFICIENT_AMOUNT).is_ok());
        assert!(transfer(&mut blockchain, account_1, account_2, SUFFICIENT_AMOUNT).is_err());
    }

    #[test]
    fn test_ok_when_high_stake_put_after_receiving() {
        let mut blockchain = Blockchain::new();
        println!("Genesis block: {:?}", blockchain.chain[0]);

        let account_1 = "Allice".to_string();
        let account_2 = "Bob".to_string();
        initiate_account(&mut blockchain, account_1.clone());
        initiate_account(&mut blockchain, account_2.clone());

        assert!(transfer(
            &mut blockchain,
            account_2,
            account_1.clone(),
            SUFFICIENT_AMOUNT
        )
        .is_ok());
        assert!(put_stake(&mut blockchain, account_1, EXCEESIVE_AMOUNT).is_ok());
    }
}
