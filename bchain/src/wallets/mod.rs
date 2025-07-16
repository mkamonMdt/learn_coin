use crate::{
    config::config_utils,
    primitives::{PendingUnstake, Wallet},
};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Wallets {
    pub wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn stake(&mut self, user: &str, amount: f64, fee: f64) -> Result<(), String> {
        let wallet = self.wallets.get_mut(user).ok_or("User not found")?;
        if wallet.balance < amount + fee {
            return Err("Insufficient ballance to stake".to_string());
        }
        wallet.balance -= amount + fee;
        wallet.staked += amount;
        Ok(())
    }

    pub fn unstake(
        &mut self,
        user: &str,
        block_height: usize,
        amount: f64,
        fee: f64,
    ) -> Result<(), String> {
        let unstake_epoch = config_utils::get_epoch(block_height) + 2;
        let wallet = self.wallets.get_mut(user).ok_or("User not found")?;
        if wallet.staked < amount {
            return Err("Insufficient stake to unstake".to_string());
        }
        if wallet.balance < fee {
            return Err(format!("Insufficient  balance for fee: {}", user));
        }
        wallet.balance -= fee;
        wallet.staked -= amount;
        wallet.pending_unstakes.push_back(PendingUnstake {
            amount,
            effective_epoch: unstake_epoch,
        });
        Ok(())
    }

    pub fn transfer(
        &mut self,
        sender: &str,
        receiver: &str,
        amount: f64,
        fee: f64,
    ) -> Result<(), String> {
        let sender_wallet = self.wallets.get_mut(sender).ok_or("Sender not found")?;
        if sender_wallet.balance < amount + fee {
            return Err("Insufficient balance".to_string());
        }
        sender_wallet.balance -= amount + fee;
        let receiver_wallet = self
            .wallets
            .entry(receiver.to_string())
            .or_insert(Wallet::new(0.));
        receiver_wallet.balance += amount;
        Ok(())
    }
}

impl Wallets {
    pub fn return_stakes(&mut self, epoch: usize) {
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

    pub fn get_stake_pool(&mut self) -> HashMap<String, f64> {
        let mut stake_pool = HashMap::new();
        for (user, wallet) in &self.wallets {
            if wallet.staked > 0.0 {
                stake_pool.insert(user.clone(), wallet.staked);
            }
        }
        stake_pool
    }
}
