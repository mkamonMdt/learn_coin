pub mod bchain_error;
pub mod message;
pub mod primitives;

mod chain;
mod config;
mod contracts;
mod patricia_merkle_trie;
mod validators;
mod wallets;

use chain::Chain;
use config::{config_utils, static_config};
use patricia_merkle_trie::state_root;
use primitives::*;
use std::collections::HashMap;
use validators::TwoEpochValidators;
use wallets::Wallets;

#[derive(Debug)]
pub struct Blockchain {
    chain: Chain,
    pub wallets: Wallets,
    contracts: HashMap<String, Vec<u8>>,
    contract_storage: HashMap<String, HashMap<String, Vec<u8>>>,
    validators: TwoEpochValidators,
    total_staked: f64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut wallets = Wallets::default();
        wallets
            .wallets
            .insert(static_config::GENESIS.to_string(), Wallet::new(1000.));
        let (state_root, _) = state_root::compute(&wallets);
        Blockchain {
            chain: Chain::new(state_root),
            wallets,
            contracts: HashMap::new(),
            contract_storage: HashMap::new(),
            validators: TwoEpochValidators::new(static_config::EPOCH_HEIGHT),
            total_staked: 0.0,
        }
    }

    fn get_epoch_seed(&self, epoch: usize) -> String {
        match config_utils::get_validators_consensus_block(epoch) {
            x if x < 2 => x.to_string(),
            validators_consensus_block => {
                assert!(
                    validators_consensus_block < self.chain.len(),
                    "Chain of len={} too short for epoch={}",
                    validators_consensus_block,
                    epoch
                );
                self.chain
                    .get_block_by_idx(validators_consensus_block)
                    .unwrap()
                    .hash
                    .clone()
            }
        }
    }

    fn return_stakes(wallets: &mut Wallets, epoch: usize) {
        for wallet in wallets.wallets.values_mut() {
            while let Some(pending) = wallet.pending_unstakes.front() {
                if pending.effective_epoch <= epoch {
                    wallet.balance += wallet.pending_unstakes.pop_front().unwrap().amount;
                } else {
                    break;
                }
            }
        }
    }

    fn get_stake_pool(wallets: &Wallets) -> HashMap<String, f64> {
        let mut stake_pool = HashMap::new();
        for (user, wallet) in &wallets.wallets {
            if wallet.staked > 0.0 {
                stake_pool.insert(user.clone(), wallet.staked);
            }
        }
        stake_pool
    }

    fn distribute_rewards(&mut self) {
        if self.total_staked == 0.0 {
            return;
        }

        let total_reward = self.total_staked * static_config::REWARD_RATE_PER_EPOCH;
        for user in self.validators.get_current_epoch_validators() {
            let wallet = self.wallets.wallets.get_mut(user).unwrap();
            let user_reward = (wallet.staked / self.total_staked) * total_reward;
            wallet.balance += user_reward;
        }
    }

    fn on_first_block_of_epoch(&mut self) {
        let block_height = self.chain.len();
        let is_epochs_first_block = (block_height % static_config::EPOCH_HEIGHT) == 0;
        if !is_epochs_first_block {
            return;
        }

        let epoch = config_utils::get_epoch(block_height);
        let next_epoch = epoch + 1;
        let seed = self.get_epoch_seed(next_epoch);
        self.distribute_rewards();
        let stake_pool = Self::get_stake_pool(&self.wallets);
        self.total_staked = stake_pool.values().sum();
        self.validators.update_validators(&stake_pool, seed);
        Self::return_stakes(&mut self.wallets, epoch);
    }

    fn process_block(&mut self, block: Block) -> Result<(), String> {
        self.on_first_block_of_epoch();
        if block.hash != block.calculate_hash() {
            return Err("Block hash corrupted".to_string());
        }
        if let Some(prev_block) = self.chain.get_last_block() {
            if prev_block.hash != block.previous_hash {
                return Err(format!(
                    "Block's previous hash does not match current tip at height={}",
                    self.chain.len()
                ));
            }
        }
        self.add_block(block.transactions)
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), String> {
        let block_height = self.chain.len();
        let slot_in_epoch = block_height % static_config::EPOCH_HEIGHT;
        let validator = self
            .validators
            .get_current_epoch_validators()
            .get(slot_in_epoch)
            .ok_or("No validators available")?
            .clone();
        let previous_block = self.chain.get_last_block().unwrap().clone();

        for tx in &transactions {
            match &tx.tx_type {
                TransactionType::Stake { user, amount } => {
                    let wallet = self.wallets.wallets.get_mut(user).ok_or("User not found")?;
                    if wallet.balance < *amount + tx.fee {
                        return Err("Insufficient ballance to stake".to_string());
                    }
                    wallet.balance -= *amount + tx.fee;
                    wallet.staked += *amount;
                }
                TransactionType::Unstake { user, amount } => {
                    let unstake_epoch = config_utils::get_epoch(block_height) + 2;
                    let wallet = self.wallets.wallets.get_mut(user).ok_or("User not found")?;
                    if wallet.staked < *amount {
                        return Err("Insufficient stake to unstake".to_string());
                    }
                    if wallet.balance < tx.fee {
                        return Err(format!("Insufficient  balance for fee: {}", user));
                    }
                    wallet.balance -= tx.fee;
                    wallet.staked -= *amount;
                    wallet.pending_unstakes.push_back(PendingUnstake {
                        amount: *amount,
                        effective_epoch: unstake_epoch,
                    });
                }
                TransactionType::Transfer {
                    sender,
                    receiver,
                    amount,
                } => {
                    let sender_wallet = self
                        .wallets
                        .wallets
                        .get_mut(sender)
                        .ok_or("Sender not found")?;
                    if sender_wallet.balance < *amount + tx.fee {
                        return Err("Insufficient balance".to_string());
                    }
                    sender_wallet.balance -= *amount + tx.fee;
                    let receiver_wallet = self
                        .wallets
                        .wallets
                        .entry(receiver.clone())
                        .or_insert(Wallet::new(0.));
                    receiver_wallet.balance += *amount;
                }
                TransactionType::DeployContract { code } => {
                    let contract_address = format!("contract_{}", self.contracts.len());
                    self.contracts
                        .insert(contract_address.clone(), code.clone());
                    println!("Deployed contract at address: {}", contract_address);
                }
                TransactionType::CallContract { contract_address } => {
                    // Deduct the fee from the sender (Alice)
                    let sender_wallet = self
                        .wallets
                        .wallets
                        .get_mut(&tx.sender)
                        .ok_or("Sender not found")?;
                    if sender_wallet.balance < tx.fee {
                        return Err("Insufficient balance for fee".to_string());
                    }
                    sender_wallet.balance -= tx.fee;
                    let x = &self
                        .contracts
                        .get(contract_address)
                        .ok_or("Contract not found")?
                        .clone();
                    contracts::execute(self, x, contract_address, &tx.sender)?;
                }
            }
        }

        let (state_root, _) = state_root::compute(&self.wallets);
        let new_block = Block::new(
            transactions.clone(),
            previous_block.hash.clone(),
            validator.clone(),
            state_root,
        );
        let validator_wallet = self.wallets.wallets.get_mut(&validator).unwrap();
        validator_wallet.balance += new_block.total_fees;

        self.chain.push(new_block);
        Ok(())
    }

    fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain.get_block_by_idx(i).unwrap();
            let previous = &self.chain.get_block_by_idx(i - 1).unwrap();

            if current.hash != current.calculate_hash() || current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
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
            .add_block(vec![Transaction::new(
                static_config::GENESIS.to_string(),
                TransactionType::Transfer {
                    sender: static_config::GENESIS.to_string(),
                    receiver: user,
                    amount: INITIAL_AMOUNT,
                },
                0.0,
            )])
            .unwrap();
    }

    fn put_stake(blockchain: &mut Blockchain, user: String, amount: f64) -> Result<(), String> {
        blockchain.add_block(vec![Transaction::new(
            user.clone(),
            TransactionType::Stake { user, amount },
            0.0,
        )])
    }

    fn transfer(
        blockchain: &mut Blockchain,
        sender: String,
        receiver: String,
        amount: f64,
    ) -> Result<(), String> {
        blockchain.add_block(vec![Transaction::new(
            sender.clone(),
            TransactionType::Transfer {
                sender,
                receiver,
                amount,
            },
            0.0,
        )])
    }

    #[test]
    fn test_genesis_block() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        let first_block = blockchain.chain.get_block_by_idx(0).unwrap();
        assert_eq!(first_block.previous_hash, "0".to_owned());
        assert_eq!(first_block.validator, static_config::GENESIS.to_owned());
    }

    #[test]
    fn test_ok_when_put_valid_stake() {
        let mut blockchain = Blockchain::new();
        println!("Genesis block: {:?}", blockchain.chain.get_block_by_idx(0));

        let account_1 = "Allice".to_string();
        initiate_account(&mut blockchain, account_1.clone());
        assert!(put_stake(&mut blockchain, account_1, SUFFICIENT_AMOUNT).is_ok());
    }

    #[test]
    fn test_error_when_put_too_high_stake() {
        let mut blockchain = Blockchain::new();
        println!(
            "Genesis block: {:?}",
            blockchain.chain.get_block_by_idx(0).unwrap()
        );

        let account_1 = "Allice".to_string();
        initiate_account(&mut blockchain, account_1.clone());
        assert!(put_stake(&mut blockchain, account_1, EXCEESIVE_AMOUNT).is_err());
    }

    #[test]
    fn test_error_when_too_high_stake_put_after_tx() {
        let mut blockchain = Blockchain::new();
        println!("Genesis block: {:?}", blockchain.chain.get_block_by_idx(0));

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
        println!("Genesis block: {:?}", blockchain.chain.get_block_by_idx(0));

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
        println!("Genesis block: {:?}", blockchain.chain.get_block_by_idx(0));

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
        assert_eq!(config_utils::get_validators_consensus_block(0), 0);
        assert_eq!(config_utils::get_validators_consensus_block(1), 0);
        assert_eq!(
            config_utils::get_validators_consensus_block(2),
            static_config::EPOCH_HEIGHT - 1
        );
    }
}
