use crate::Blockchain;
use wasmi::{Caller, Extern};

pub fn store(
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
