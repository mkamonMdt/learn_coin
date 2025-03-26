pub mod primitives;

use primitives::{block::Block, opcodes_language::Opcode, transaction::*};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, VecDeque},
    vec,
};

const EPOCH_HEIGHT: usize = 10;
const BLOCK_CHAIN_WORTH: f64 = 1000.0;
const GENESIS: &str = "Genesis";
const REWARD_RATE_PER_EPOCH: f64 = 0.00001;

#[derive(Debug, Serialize)]
struct PendingUnstake {
    amount: f64,
    effective_epoch: usize,
}

#[derive(Debug, Serialize)]
struct Wallet {
    balance: f64,
    staked: f64,
    pending_unstakes: VecDeque<PendingUnstake>,
}

#[derive(Debug)]
struct ContractState {
    storage: HashMap<String, f64>,
}

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    wallets: HashMap<String, Wallet>,
    contracts: HashMap<String, (Vec<Opcode>, ContractState)>,
    slots_per_epoch: usize,
    current_epoch_validators: Vec<String>,
    next_epoch_validators: Vec<String>,
    total_staked: f64,
}

impl Blockchain {
    fn new() -> Self {
        let mut wallets = HashMap::new();
        wallets.insert(
            GENESIS.to_string(),
            Wallet {
                balance: 1000.0,
                staked: 0.0,
                pending_unstakes: VecDeque::new(),
            },
        );
        let (state_root, _) = Self::compute_state_root(&wallets);
        let genesis_block = Block::new(
            vec![Transaction::new(
                TransactionType::Transfer {
                    sender: GENESIS.to_string(),
                    receiver: "System".to_string(),
                    amount: BLOCK_CHAIN_WORTH,
                },
                0.0,
            )],
            "0".to_string(),
            GENESIS.to_string(),
            state_root,
        );
        Blockchain {
            chain: vec![genesis_block],
            wallets,
            contracts: HashMap::new(),
            slots_per_epoch: EPOCH_HEIGHT,
            current_epoch_validators: vec![GENESIS.to_string(); EPOCH_HEIGHT],
            next_epoch_validators: vec![GENESIS.to_string(); EPOCH_HEIGHT],
            total_staked: 0.0,
        }
    }

    fn compute_state_root(wallets: &HashMap<String, Wallet>) -> (String, Vec<Vec<String>>) {
        if wallets.is_empty() {
            let zero_hash = format!("{:x}", Sha256::new().finalize());
            return (zero_hash, vec![]);
        }

        let mut leaves: Vec<(String, String)> = wallets
            .iter()
            .map(|(user, wallet)| {
                let data = format!("{}{}", user, serde_json::to_string(wallet).unwrap());
                let mut hasher = Sha256::new();
                hasher.update(data);
                (user.clone(), format!("{:x}", hasher.finalize()))
            })
            .collect();
        leaves.sort_by(|a, b| a.0.cmp(&b.0));
        let mut tree: Vec<Vec<String>> = vec![leaves.iter().map(|(_, h)| h.clone()).collect()];

        let mut current_level = tree[0].clone();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    chunk[0].to_string()
                };
                let mut hasher = Sha256::new();
                hasher.update(&combined);
                next_level.push(format!("{:x}", hasher.finalize()));
            }
            tree.push(next_level.clone());
            current_level = next_level;
        }
        (current_level[0].clone(), tree)
    }

    fn get_merkle_proof(&self, user: &str) -> Option<Vec<(String, bool)>> {
        let (_, tree) = Self::compute_state_root(&self.wallets);
        if tree.is_empty() {
            return None;
        }

        // Find the leaf index for the user
        let leaves = &tree[0];
        let leaf_data = format!(
            "{}{}",
            user,
            serde_json::to_string(self.wallets.get(user)?).unwrap()
        );
        let mut hasher = Sha256::new();
        hasher.update(&leaf_data);
        let leaf_hash = format!("{:x}", hasher.finalize());
        let leaf_idx = leaves.iter().position(|h| *h == leaf_hash)?;

        // Build proof by collection siblings
        let mut proof = Vec::new();
        let mut idx = leaf_idx;
        for level in &tree[..tree.len() - 1] {
            let is_left = idx % 2 == 0;
            let sibling_idx = if is_left { idx + 1 } else { idx - 1 };
            if sibling_idx < level.len() {
                proof.push((level[sibling_idx].clone(), is_left))
            }
            idx /= 2; //Move up to parent
        }
        Some(proof)
    }

    fn verify_merkle_proof(
        &self,
        user: &str,
        proof: &[(String, bool)],
        block_height: usize,
    ) -> bool {
        let block = self.chain.get(block_height).unwrap();
        let wallet = match self.wallets.get(user) {
            Some(w) => w,
            None => return false,
        };

        //compute leaf hash
        let leaf_data = format!("{}{}", user, serde_json::to_string(wallet).unwrap());
        let mut hasher = Sha256::new();
        hasher.update(&leaf_data);
        let mut current_hash = format!("{:x}", hasher.finalize());

        //Recompute root using proof
        for (sibling, is_left) in proof {
            let combined = if *is_left {
                format!("{}{}", current_hash, sibling)
            } else {
                format!("{}{}", sibling, current_hash)
            };
            let mut hasher = Sha256::new();
            hasher.update(&combined);
            current_hash = format!("{:x}", hasher.finalize());
        }

        current_hash == block.state_root
    }

    fn get_stake_pool(&self) -> HashMap<String, f64> {
        let mut stake_pool = HashMap::new();
        for (user, wallet) in &self.wallets {
            if wallet.staked > 0.0 {
                stake_pool.insert(user.clone(), wallet.staked);
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
        let stake_pool = self.get_stake_pool();
        self.total_staked = stake_pool.values().sum();

        for slot_in_epoch in 0..self.next_epoch_validators.len() {
            self.next_epoch_validators[slot_in_epoch] =
                self.get_validator_for_slots(&stake_pool, next_epoch, slot_in_epoch);
        }
    }

    fn return_stakes(&mut self, block_height: usize) {
        let epoch = self.get_epoch(block_height);
        for wallet in self.wallets.values_mut() {
            while let Some(pending) = wallet.pending_unstakes.front() {
                if pending.effective_epoch <= epoch {
                    wallet.balance += wallet.pending_unstakes.pop_front().unwrap().amount;
                } else {
                    break;
                }
            }
        }
    }

    fn distribute_rewards(&mut self) {
        if self.total_staked == 0.0 {
            return;
        }

        let total_reward = self.total_staked * REWARD_RATE_PER_EPOCH;
        for user in &self.current_epoch_validators {
            let wallet = self.wallets.get_mut(user).unwrap();
            let user_reward = (wallet.staked / self.total_staked) * total_reward;
            wallet.balance += user_reward;
        }
    }

    fn on_first_block_of_epoch(&mut self) {
        let block_height = self.chain.len();
        let is_epochs_first_block = (block_height % self.slots_per_epoch) == 0;
        if !is_epochs_first_block {
            return;
        }

        self.distribute_rewards();
        self.update_validators(block_height);
        self.return_stakes(block_height);
    }

    fn execute_contract(&mut self, code: &[Opcode], contract_address: &str) -> Result<(), String> {
        let mut stack: Vec<f64> = Vec::new();
        let contract_state = self
            .contracts
            .get_mut(contract_address)
            .map(|(_, state)| state)
            .ok_or("Contract not found")?;

        for op in code {
            match op {
                Opcode::Push(value) => stack.push(*value),
                Opcode::Add => {
                    if stack.len() < 2 {
                        return Err("Stack underflow: Add".to_string());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a + b);
                }
                Opcode::Sub => {
                    if stack.len() < 2 {
                        return Err("Stack underflow: Sub".to_string());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a - b);
                }
                Opcode::Eq => {
                    if stack.len() < 2 {
                        return Err("Stack underflow: Eq".to_string());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(if a == b { 1.0 } else { 0.0 });
                }
                Opcode::Store(key) => {
                    if stack.is_empty() {
                        return Err("Stack underflow: Store".to_string());
                    }
                    let value = stack.pop().unwrap();
                    contract_state.storage.insert(key.clone(), value);
                }
                Opcode::Load(key) => {
                    let value = *contract_state.storage.get(key).unwrap_or(&0.0);
                    stack.push(value);
                }
                Opcode::Balance(user) => {
                    let balance = self.wallets.get(user).map(|w| w.balance).unwrap_or(0.0);
                    stack.push(balance);
                }
                Opcode::Transfer(from, to) => {
                    if stack.is_empty() {
                        return Err("Stack underflow: Transfer".to_string());
                    }
                    let amount = stack.pop().unwrap();
                    if amount <= 0.0 {
                        return Err("Invalid transfer amount".to_string());
                    }
                    let from_wallet = self
                        .wallets
                        .get_mut(from)
                        .ok_or(format!("User not found {}", from))?;
                    from_wallet.balance -= amount;
                    let to_wallet = self.wallets.entry(to.clone()).or_insert(Wallet {
                        balance: 0.0,
                        staked: 0.0,
                        pending_unstakes: VecDeque::new(),
                    });
                    to_wallet.balance += amount;
                }
            }
        }
        Ok(())
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

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), String> {
        let block_height = self.chain.len();
        let slot_in_epoch = block_height % self.slots_per_epoch;
        let validator = self
            .current_epoch_validators
            .get(slot_in_epoch)
            .ok_or("No validators available")?
            .clone();
        let previous_block = self.chain.last().unwrap().clone();

        for tx in &transactions {
            match &tx.tx_type {
                TransactionType::Stake { user, amount } => {
                    let wallet = self.wallets.get_mut(user).ok_or("User not found")?;
                    if wallet.balance < *amount + tx.fee {
                        return Err("Insufficient ballance to stake".to_string());
                    }
                    wallet.balance -= *amount + tx.fee;
                    wallet.staked += *amount;
                }
                TransactionType::Unstake { user, amount } => {
                    let unstake_epoch = self.get_epoch(block_height) + 2;
                    let wallet = self.wallets.get_mut(user).ok_or("User not found")?;
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
                    let sender_wallet = self.wallets.get_mut(sender).ok_or("Sender not found")?;
                    if sender_wallet.balance < *amount + tx.fee {
                        return Err("Insufficient balance".to_string());
                    }
                    sender_wallet.balance -= *amount + tx.fee;
                    let receiver_wallet = self.wallets.entry(receiver.clone()).or_insert(Wallet {
                        balance: 0.0,
                        staked: 0.0,
                        pending_unstakes: VecDeque::new(),
                    });
                    receiver_wallet.balance += *amount;
                }
                TransactionType::DeployContract { code } => {
                    let contract_address = format!("contract_{}", self.contracts.len());
                    self.contracts.insert(
                        contract_address.clone(),
                        (
                            code.clone(),
                            ContractState {
                                storage: HashMap::new(),
                            },
                        ),
                    );
                    println!("Deployed contract at address: {}", contract_address);
                }
                TransactionType::CallContract { contract_address } => {
                    let x = &self
                        .contracts
                        .get(contract_address)
                        .ok_or("Contract not found")?
                        .0
                        .clone();
                    self.execute_contract(x, contract_address)?;
                }
            }
        }

        let (state_root, _) = Self::compute_state_root(&self.wallets);
        let new_block = Block::new(
            transactions.clone(),
            previous_block.hash.clone(),
            validator.clone(),
            state_root,
        );
        let validator_wallet = self.wallets.get_mut(&validator).unwrap();
        validator_wallet.balance += new_block.total_fees;

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
            .add_block(vec![Transaction::new(
                TransactionType::Transfer {
                    sender: GENESIS.to_string(),
                    receiver: user,
                    amount: INITIAL_AMOUNT,
                },
                0.0,
            )])
            .unwrap();
    }

    fn put_stake(blockchain: &mut Blockchain, user: String, amount: f64) -> Result<(), String> {
        blockchain.add_block(vec![Transaction::new(
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

    #[test]
    fn test_simple_contract() {
        let mut blockchain = Blockchain::new();

        blockchain.wallets.insert(
            "Alice".to_string(),
            Wallet {
                balance: 500.0,
                staked: 0.0,
                pending_unstakes: VecDeque::new(),
            },
        );

        // Deploy a simple contract: "If Alice's balance > 100, transfer 50 to Bob"
        let contract_code = vec![
            Opcode::Balance("Alice".to_string()),   // Push Alice's balance
            Opcode::Push(100.0),                    // Push 100
            Opcode::Sub,                            // Subtract: balance - 100
            Opcode::Push(0.0),                      // Push 0
            Opcode::Eq,        // Check if (balance - 100) == 0 (i.e., balance <= 100)
            Opcode::Push(0.0), // Push 0
            Opcode::Eq,        // If balance > 100, stack has 1, else 0
            Opcode::Store("condition".to_string()), // Store the condition result
            Opcode::Load("condition".to_string()), // Load the condition
            Opcode::Push(1.0), // Push 1
            Opcode::Eq,        // Check if condition == 1
            Opcode::Push(50.0), // Push 50 (amount to transfer)
            Opcode::Transfer("Alice".to_string(), "Bob".to_string()), // Transfer 50 from Alice to Bob
        ];

        let tx1 = Transaction::new(
            TransactionType::DeployContract {
                code: contract_code,
            },
            1.0,
        );
        blockchain.add_block(vec![tx1]).unwrap();

        // Call the contract
        let tx2 = Transaction::new(
            TransactionType::CallContract {
                contract_address: "contract_0".to_string(),
            },
            1.0,
        );
        assert!(blockchain.add_block(vec![tx2]).is_ok());
    }
}
