use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};

const EPOCH_HEIGHT: usize = 10;
const BLOCK_CHAIN_WORTH: f64 = 1000.0;
const GENESIS: &str = "Genesis";

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
    Unstake {
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
struct PendingUnstake {
    user: String,
    amount: f64,
    effective_epoch: usize,
}

#[derive(Debug)]
struct Blockchain {
    chain: Vec<Block>,
    wallets: HashMap<String, Wallet>,
    slots_per_epoch: usize,
    current_epoch_validators: Vec<String>,
    next_epoch_validators: Vec<String>,
    pending_unstakes: VecDeque<PendingUnstake>,
}

impl Blockchain {
    fn new() -> Self {
        let mut wallets = HashMap::new();
        wallets.insert(GENESIS.to_string(), Wallet { balance: 1000.0 });
        let genesis_block = Block::new(
            vec![Transaction {
                tx_type: TransactionType::Transfer {
                    sender: GENESIS.to_string(),
                    receiver: "System".to_string(),
                    amount: BLOCK_CHAIN_WORTH,
                },
            }],
            "0".to_string(),
            GENESIS.to_string(),
        );
        Blockchain {
            chain: vec![genesis_block],
            wallets,
            slots_per_epoch: EPOCH_HEIGHT,
            current_epoch_validators: vec![GENESIS.to_string(); EPOCH_HEIGHT],
            next_epoch_validators: vec![GENESIS.to_string(); EPOCH_HEIGHT],
            pending_unstakes: VecDeque::new(),
        }
    }

    fn get_stake_pool(&self, up_to_block: usize) -> HashMap<String, f64> {
        let mut stake_pool = HashMap::new();
        let up_to_block = up_to_block.min(self.chain.len() - 1);
        for block in &self.chain[..=up_to_block] {
            for tx in &block.transactions {
                match &tx.tx_type {
                    TransactionType::Stake { user, amount } => {
                        *stake_pool.entry(user.clone()).or_insert(0.0) += amount;
                    }
                    TransactionType::Unstake { user, amount } => {
                        let current_stake = stake_pool.entry(user.clone()).or_insert(0.0);
                        *current_stake -= amount;
                        if *current_stake <= 0.0 {
                            stake_pool.remove(user);
                        }
                    }
                    _ => {}
                }
            }
        }
        stake_pool
    }

    fn get_epoch(&self, block_height: usize) -> usize {
        block_height / self.slots_per_epoch
    }

    fn get_validators_consensus_block(&self, epoch: usize) -> usize {
        if epoch < 2 {
            0
        } else {
            (epoch - 1) * self.slots_per_epoch - 1
        }
    }

    fn get_epoch_seed(&self, epoch: usize) -> String {
        match self.get_validators_consensus_block(epoch) {
            x if x < 2 => x.to_string(),
            validators_consensus_block => {
                assert!(
                    validators_consensus_block < self.chain.len(),
                    "Chain of len={} too short for epoch={}",
                    validators_consensus_block,
                    epoch
                );
                self.chain[validators_consensus_block].hash.clone()
            }
        }
    }

    fn get_validator_for_slots(
        &self,
        stake_pool: &HashMap<String, f64>,
        epoch: usize,
        slot: usize,
    ) -> String {
        let total_stake: f64 = stake_pool.values().sum();
        if total_stake == 0.0 {
            return GENESIS.to_string();
        }

        let seed = self.get_epoch_seed(epoch);
        let slot_seed = format!("{}{}", seed, slot);
        let mut hasher = Sha256::new();
        hasher.update(&slot_seed);
        let result = hasher.finalize();
        let seed_value = u64::from_le_bytes(result[..8].try_into().unwrap());
        let random_point = seed_value as f64 % total_stake;

        let mut cumulative = 0.0;
        for (user, stake) in stake_pool {
            cumulative += stake;
            if cumulative >= random_point {
                return user.clone();
            }
        }
        GENESIS.to_string()
    }

    fn update_validators(&mut self, block_height: usize) {
        std::mem::swap(
            &mut self.current_epoch_validators,
            &mut self.next_epoch_validators,
        );
        let next_epoch = self.get_epoch(block_height) + 1;
        let up_to_block = self.get_validators_consensus_block(next_epoch);
        let stake_pool = self.get_stake_pool(up_to_block);

        for slot_in_epoch in 0..self.next_epoch_validators.len() {
            self.next_epoch_validators[slot_in_epoch] =
                self.get_validator_for_slots(&stake_pool, next_epoch, slot_in_epoch);
        }
    }

    fn return_stakes(&mut self, block_height: usize) {
        let epoch = self.get_epoch(block_height);
        while let Some(pending) = self.pending_unstakes.front() {
            if pending.effective_epoch <= epoch {
                let unstake = self.pending_unstakes.pop_front().unwrap();
                let wallet = self.wallets.get_mut(&unstake.user).unwrap();
                wallet.balance += unstake.amount;
            } else {
                break;
            }
        }
    }

    fn on_first_block_of_epoch(&mut self) {
        let block_height = self.chain.len();
        let is_epochs_first_block = (block_height % self.slots_per_epoch) == 0;
        if !is_epochs_first_block {
            return;
        }

        self.update_validators(block_height);
        self.return_stakes(block_height);
    }

    fn process_block(&mut self, block: Block) -> Result<(), String> {
        self.on_first_block_of_epoch();
        if block.hash != block.calculate_hash() {
            return Err("Block hash corrupted".to_string());
        }
        if let Some(prev_block) = self.chain.last() {
            if prev_block.hash != block.previous_hash {
                return Err(format!(
                    "Block's previous hash does not match current tip at height={}",
                    self.chain.len()
                ));
            }
        }
        self.add_block(block.transactions)
    }

    fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), String> {
        let block_height = self.chain.len();
        let slot_in_epoch = block_height % self.slots_per_epoch;
        let validator = self
            .current_epoch_validators
            .get(slot_in_epoch)
            .ok_or("No validators available")?;
        let previous_block = self.chain.last().unwrap();
        let new_block = Block::new(
            transactions.clone(),
            previous_block.hash.clone(),
            validator.clone(),
        );

        let stake_pool = self.get_stake_pool(block_height);
        for tx in &transactions {
            match &tx.tx_type {
                TransactionType::Stake { user, amount } => {
                    let wallet = self.wallets.get_mut(user).ok_or("User not found")?;
                    if wallet.balance < *amount {
                        return Err("Insufficient ballance to stake".to_string());
                    }
                    wallet.balance -= *amount;
                }
                TransactionType::Unstake { user, amount } => {
                    let current_stake = stake_pool.get(user).ok_or("User not found")?;
                    if *current_stake < *amount {
                        return Err("Insufficient stake to unstake".to_string());
                    }
                    self.pending_unstakes.push_back(PendingUnstake {
                        user: user.clone(),
                        amount: *amount,
                        effective_epoch: self.get_epoch(block_height) + 2,
                    });
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

        let validator_wallet = self.wallets.get_mut(validator).unwrap();
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
                    sender: GENESIS.to_string(),
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
        assert_eq!(first_block.validator, GENESIS.to_owned());
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

    #[test]
    fn test_validator_consensus_block() {
        let blockchain = Blockchain::new();

        assert_eq!(blockchain.get_validators_consensus_block(0), 0);
        assert_eq!(blockchain.get_validators_consensus_block(1), 0);
        assert_eq!(
            blockchain.get_validators_consensus_block(2),
            EPOCH_HEIGHT - 1
        );
    }
}
