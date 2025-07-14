use crate::Blockchain;
use wasmi::{Caller, Extern};

pub fn load(
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
