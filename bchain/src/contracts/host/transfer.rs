use crate::primitives::Wallet;
use crate::Blockchain;
use wasmi::{Caller, Extern};

pub fn transfer(
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
