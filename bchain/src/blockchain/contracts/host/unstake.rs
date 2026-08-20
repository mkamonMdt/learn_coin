use crate::config::config_utils;
use crate::primitives::PendingUnstake;
use crate::Blockchain;
use wasmi::Caller;

pub fn unstake(
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
