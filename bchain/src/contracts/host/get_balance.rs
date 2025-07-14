use crate::Blockchain;
use wasmi::{Caller, Extern};

pub fn get_balance(
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
