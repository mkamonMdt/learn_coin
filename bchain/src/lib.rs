pub mod bchain_error;
pub mod message;
pub mod primitives;
pub mod wallets;

mod config;
mod patricia_merkle_trie;

use config::{config_utils, static_config};
use patricia_merkle_trie::state_root;
use primitives::{block::Block, transaction::*};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, vec};
use wallets::{PendingUnstake, Wallet, Wallets};
use wasmi::{Caller, Engine, Extern, Func, Linker, Module, Store};

#[derive(Debug)]
struct ContractState {
    storage: HashMap<String, f64>,
}

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub wallets: Wallets,
    contracts: HashMap<String, Vec<u8>>,
    contract_storage: HashMap<String, HashMap<String, Vec<u8>>>,
    current_epoch_validators: Vec<String>,
    next_epoch_validators: Vec<String>,
    total_staked: f64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut wallets = Wallets::default();
        wallets
            .wallets
            .insert(static_config::GENESIS.to_string(), Wallet::new(1000.));
        let (state_root, _) = state_root::compute(&wallets);
        let genesis_block = Block::new(
            vec![Transaction::new(
                static_config::GENESIS.to_string(),
                TransactionType::Transfer {
                    sender: static_config::GENESIS.to_string(),
                    receiver: "System".to_string(),
                    amount: static_config::BLOCK_CHAIN_WORTH,
                },
                0.0,
            )],
            "0".to_string(),
            static_config::GENESIS.to_string(),
            state_root,
        );
        Blockchain {
            chain: vec![genesis_block],
            wallets,
            contracts: HashMap::new(),
            contract_storage: HashMap::new(),
            current_epoch_validators: vec![
                static_config::GENESIS.to_string();
                static_config::EPOCH_HEIGHT
            ],
            next_epoch_validators: vec![
                static_config::GENESIS.to_string();
                static_config::EPOCH_HEIGHT
            ],
            total_staked: 0.0,
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
                self.chain[validators_consensus_block].hash.clone()
            }
        }
    }

    fn get_validator_for_slots(
        stake_pool: &HashMap<String, f64>,
        seed: String,
        slot: usize,
    ) -> String {
        let total_stake: f64 = stake_pool.values().sum();
        if total_stake == 0.0 {
            return static_config::GENESIS.to_string();
        }

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
        static_config::GENESIS.to_string()
    }

    fn update_validators(&mut self, current_epoch: usize) {
        std::mem::swap(
            &mut self.current_epoch_validators,
            &mut self.next_epoch_validators,
        );
        let stake_pool = Self::get_stake_pool(&self.wallets);
        let next_epoch = current_epoch + 1;
        let seed = self.get_epoch_seed(next_epoch);
        self.total_staked = stake_pool.values().sum();

        for slot_in_epoch in 0..self.next_epoch_validators.len() {
            self.next_epoch_validators[slot_in_epoch] =
                Self::get_validator_for_slots(&stake_pool, seed.clone(), slot_in_epoch);
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

    fn distribute_rewards(&mut self) {
        if self.total_staked == 0.0 {
            return;
        }

        let total_reward = self.total_staked * static_config::REWARD_RATE_PER_EPOCH;
        for user in &self.current_epoch_validators {
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
        self.distribute_rewards();
        self.update_validators(epoch);
        Self::return_stakes(&mut self.wallets, epoch);
    }

    fn execute_contract(
        &mut self,
        code: &[u8],
        contract_address: &str,
        sender: &str,
    ) -> Result<(), String> {
        // Initialize the Wasm engine and store
        let engine = Engine::default();
        let module = Module::new(&engine, code)
            .map_err(|e| format!("Failed to laod Wasm module: {:?}", e))?;
        let mut store: Store<(String, String)> =
            Store::new(&engine, (contract_address.to_string(), sender.to_string()));

        // Create a linker and define host functions
        let mut linker: Linker<(String, String)> = Linker::new(&engine);

        let get_balance = Func::wrap(&mut store, get_balance_host);
        linker.define("env", "get_balance", get_balance).unwrap();

        let transfer = Func::wrap(&mut store, transfer_host);
        linker.define("env", "transfer", transfer).unwrap();

        let store_func = Func::wrap(&mut store, store_host);
        linker.define("env", "store", store_func).unwrap();

        let load_func = Func::wrap(&mut store, load_host);
        linker.define("env", "load", load_func).unwrap();

        let stake_func = Func::wrap(&mut store, stake_host);
        linker.define("env", "stake", stake_func).unwrap();

        let unstake_func = Func::wrap(&mut store, unstake_host);
        linker.define("env", "unstake", unstake_func).unwrap();

        let debug_func = Func::wrap(&mut store, debug_host);
        linker.define("env", "debug", debug_func).unwrap();

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate module: {:?}", e))?
            .start(&mut store)
            .map_err(|e| format!("Failed to start instance: {:?}", e))?;

        // Call the execute function, passing the blockchain pointer
        // Split the pointer into two i32s
        let blockchain_ptr = self as *mut Blockchain as usize; // Use usize to hold the full pointer
        let blockchain_ptr_low = (blockchain_ptr & 0xFFFFFFFF) as i32; // Lower 32 bits
        let blockchain_ptr_high = ((blockchain_ptr >> 32) & 0xFFFFFFFF) as i32; // Upper 32 bits

        let execute = instance
            .get_export(&store, "execute")
            .and_then(Extern::into_func)
            .ok_or("Failed to find execute function")?;
        let execute: Func = execute;
        execute
            .call(
                &mut store,
                &[
                    wasmi::Val::I32(blockchain_ptr_low), // Pass blockchain_ptr as an argument
                    wasmi::Val::I32(blockchain_ptr_high),
                ],
                &mut [wasmi::Val::I32(0)],
            )
            .map_err(|e| format!("Failed to execute contract: {:?}", e))
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
        let slot_in_epoch = block_height % static_config::EPOCH_HEIGHT;
        let validator = self
            .current_epoch_validators
            .get(slot_in_epoch)
            .ok_or("No validators available")?
            .clone();
        let previous_block = self.chain.last().unwrap().clone();

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
                    self.execute_contract(x, contract_address, &tx.sender)?;
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
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

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

fn store_host(
    caller: Caller<(String, String)>,
    blockchain_ptr_low: i32,
    blockchain_ptr_high: i32,
    key_ptr: i32,
    key_len: i32,
    value_ptr: i32,
    value_len: i32,
) -> i32 {
    let blockchain_ptr = ((blockchain_ptr_high as u64) << 32) | (blockchain_ptr_low as u32 as u64);
    if blockchain_ptr == 0 {
        println!("Error: blockchain_ptr is null");
        return 1;
    }
    let blockchain: &mut Blockchain = unsafe { &mut *(blockchain_ptr as *mut Blockchain) };

    let memory = match caller.get_export("memory").and_then(Extern::into_memory) {
        Some(mem) => mem,
        None => return 1,
    };

    let key_bytes = &memory.data(&caller)[key_ptr as usize..(key_ptr + key_len) as usize];
    let key = match String::from_utf8(key_bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => return 1,
    };

    let value_bytes = &memory.data(&caller)[value_ptr as usize..(value_ptr + value_len) as usize];
    let value = value_bytes.to_vec();

    let contract_address = caller.data().0.clone();
    let storage = blockchain
        .contract_storage
        .entry(contract_address)
        .or_default();
    storage.insert(key, value);
    0 // Success
}

fn load_host(
    mut caller: Caller<(String, String)>,
    blockchain_ptr_low: i32,
    blockchain_ptr_high: i32,
    key_ptr: i32,
    key_len: i32,
    value_ptr: i32, // New parameter: where to write the value
) -> i32 {
    let blockchain_ptr = ((blockchain_ptr_high as u64) << 32) | (blockchain_ptr_low as u32 as u64);
    if blockchain_ptr == 0 {
        println!("Error: blockchain_ptr is null");
        return -1;
    }
    let blockchain: &mut Blockchain = unsafe { &mut *(blockchain_ptr as *mut Blockchain) };
    let memory = match caller.get_export("memory").and_then(Extern::into_memory) {
        Some(mem) => mem,
        None => return -1,
    };

    // Read the key
    let key_bytes = &memory.data(&caller)[key_ptr as usize..(key_ptr + key_len) as usize];
    let key = match String::from_utf8(key_bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    // Get the contract address from the caller data
    let contract_address = caller.data().0.clone();

    // Look up the value
    let storage = blockchain.contract_storage.get(&contract_address);
    let value = match storage.and_then(|s| s.get(&key)) {
        Some(v) => v,
        None => return -1, // Key not found
    };

    // Write the value to the specified location
    if (value_ptr as usize) + value.len() > memory.data(&caller).len() {
        return -1; // Not enough space in memory
    }
    memory.data_mut(&mut caller)[value_ptr as usize..(value_ptr as usize) + value.len()]
        .copy_from_slice(value);
    value.len() as i32 // Return the length of the value
}

fn stake_host(
    caller: Caller<(String, String)>,
    blockchain_ptr_low: i32,
    blockchain_ptr_high: i32,
    amount: f64,
) -> i32 {
    let blockchain_ptr = ((blockchain_ptr_high as u64) << 32) | (blockchain_ptr_low as u32 as u64);
    if blockchain_ptr == 0 {
        println!("Error: blockchain_ptr is null");
        return 1;
    }
    let blockchain: &mut Blockchain = unsafe { &mut *(blockchain_ptr as *mut Blockchain) };
    let user = caller.data().1.clone();
    let wallet = match blockchain.wallets.wallets.get_mut(&user) {
        Some(wallet) => wallet,
        None => {
            println!("Error: User {} not found", user);
            return 1;
        }
    };
    if wallet.balance < amount {
        return 1;
    }
    wallet.balance -= amount;
    wallet.staked += amount;
    0
}

fn unstake_host(
    caller: Caller<(String, String)>,
    blockchain_ptr_low: i32,
    blockchain_ptr_high: i32,
    amount: f64,
) -> i32 {
    let blockchain_ptr = ((blockchain_ptr_high as u64) << 32) | (blockchain_ptr_low as u32 as u64);
    if blockchain_ptr == 0 {
        println!("Error: blockchain_ptr is null");
        return 1;
    }
    let blockchain: &mut Blockchain = unsafe { &mut *(blockchain_ptr as *mut Blockchain) };
    let block_height = blockchain.chain.len();
    let effective_epoch = config_utils::get_epoch(block_height) + 2;
    //let contract_address = caller.data().clone();
    let user = caller.data().1.clone();
    let wallet = match blockchain.wallets.wallets.get_mut(&user) {
        Some(wallet) => wallet,
        None => {
            println!("Error: User {} not found", user);
            return 1;
        }
    };
    if wallet.staked < amount {
        return 1;
    }
    wallet.staked -= amount;

    wallet.pending_unstakes.push_back(PendingUnstake {
        amount,
        effective_epoch,
    });
    0
}

// Host function: get_balance
fn get_balance_host(
    caller: Caller<(String, String)>,
    blockchain_ptr_low: i32,
    blockchain_ptr_high: i32,
    user_ptr: i32,
    user_len: i32,
) -> f64 {
    let blockchain_ptr = ((blockchain_ptr_high as u64) << 32) | (blockchain_ptr_low as u32 as u64);
    let blockchain: &mut Blockchain = unsafe { &mut *(blockchain_ptr as *mut Blockchain) };

    let memory = match caller.get_export("memory").and_then(Extern::into_memory) {
        Some(mem) => mem,
        None => return 0.0,
    };
    let user_bytes =
        memory.data(&caller)[user_ptr as usize..(user_ptr + user_len) as usize].to_vec();
    let user = match String::from_utf8(user_bytes) {
        Ok(u) => u,
        Err(_) => return 0.0,
    };
    blockchain
        .wallets
        .wallets
        .get(&user)
        .map(|w| w.balance)
        .unwrap_or(0.0)
}

// Host function: get_balance
fn transfer_host(
    caller: Caller<(String, String)>,
    blockchain_ptr_low: i32,
    blockchain_ptr_high: i32,
    from_ptr: i32,
    from_len: i32,
    to_ptr: i32,
    to_len: i32,
    amount: f64,
) -> i32 {
    let blockchain_ptr = ((blockchain_ptr_high as u64) << 32) | (blockchain_ptr_low as u32 as u64);
    let blockchain: &mut Blockchain = unsafe { &mut *(blockchain_ptr as *mut Blockchain) };
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or("Failed to get memory")
        .unwrap();
    let from_bytes =
        memory.data(&caller)[from_ptr as usize..(from_ptr + from_len) as usize].to_vec();
    let to_bytes = memory.data(&caller)[to_ptr as usize..(to_ptr + to_len) as usize].to_vec();
    let from = String::from_utf8(from_bytes).unwrap();
    let to = String::from_utf8(to_bytes).unwrap();

    if amount <= 0.0 {
        return 1; //Failure
    }
    let from_wallet = match blockchain.wallets.wallets.get_mut(&from) {
        Some(wallet) => wallet,
        None => return 1,
    };
    from_wallet.balance -= amount;
    let to_wallet = blockchain
        .wallets
        .wallets
        .entry(to.clone())
        .or_insert(Wallet::new(0.));
    to_wallet.balance += amount;
    0 // Success
}

fn debug_host(caller: Caller<(String, String)>, msg_ptr: i32, msg_len: i32, value: u32) {
    let memory = match caller.get_export("memory").and_then(Extern::into_memory) {
        Some(mem) => mem,
        None => {
            println!("Debug error: No memory export found");
            return;
        }
    };

    let msg_bytes = &memory.data(&caller)[msg_ptr as usize..(msg_ptr + msg_len) as usize];
    let msg = match String::from_utf8(msg_bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            println!("Debug error: Invalid UTF-8 string");
            return;
        }
    };

    println!("Contract debug: {} {}", msg, value);
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
        let first_block = blockchain.chain.first().unwrap();
        assert_eq!(first_block.previous_hash, "0".to_owned());
        assert_eq!(first_block.validator, static_config::GENESIS.to_owned());
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

        assert_eq!(config_utils::get_validators_consensus_block(0), 0);
        assert_eq!(config_utils::get_validators_consensus_block(1), 0);
        assert_eq!(
            config_utils::get_validators_consensus_block(2),
            static_config::EPOCH_HEIGHT - 1
        );
    }
}
