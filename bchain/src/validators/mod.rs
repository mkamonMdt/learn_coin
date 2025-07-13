use std::collections::HashMap;

use sha2::{Digest, Sha256};

use crate::config::static_config;

#[derive(Debug)]
pub struct TwoEpochValidators {
    current_epoch_validators: Vec<String>,
    next_epoch_validators: Vec<String>,
}

impl TwoEpochValidators {
    pub fn new(validators_per_epoch: usize) -> Self {
        Self {
            current_epoch_validators: vec![
                static_config::GENESIS.to_string();
                validators_per_epoch
            ],
            next_epoch_validators: vec![static_config::GENESIS.to_string(); validators_per_epoch],
        }
    }

    pub fn update_validators(&mut self, stake_pool: &HashMap<String, f64>, seed: String) {
        std::mem::swap(
            &mut self.current_epoch_validators,
            &mut self.next_epoch_validators,
        );

        for slot_in_epoch in 0..self.next_epoch_validators.len() {
            self.next_epoch_validators[slot_in_epoch] =
                Self::get_validator_for_slots(stake_pool, seed.clone(), slot_in_epoch);
        }
    }

    pub fn get_current_epoch_validators(&self) -> &Vec<String> {
        &self.current_epoch_validators
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
}
