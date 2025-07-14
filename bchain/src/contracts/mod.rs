mod host;

use crate::Blockchain;
use wasmi::{Engine, Extern, Func, Linker, Module, Store};

pub fn execute(
    blockchain: &mut Blockchain,
    code: &[u8],
    contract_address: &str,
    sender: &str,
) -> Result<(), String> {
    // Initialize the Wasm engine and store
    let engine = Engine::default();
    let module =
        Module::new(&engine, code).map_err(|e| format!("Failed to laod Wasm module: {:?}", e))?;
    let mut store: Store<(String, String)> =
        Store::new(&engine, (contract_address.to_string(), sender.to_string()));

    // Create a linker and define host functions
    let mut linker: Linker<(String, String)> = Linker::new(&engine);

    let get_balance = Func::wrap(&mut store, host::get_balance);
    linker.define("env", "get_balance", get_balance).unwrap();

    let transfer = Func::wrap(&mut store, host::transfer);
    linker.define("env", "transfer", transfer).unwrap();

    let store_func = Func::wrap(&mut store, host::store);
    linker.define("env", "store", store_func).unwrap();

    let load_func = Func::wrap(&mut store, host::load);
    linker.define("env", "load", load_func).unwrap();

    let stake_func = Func::wrap(&mut store, host::stake);
    linker.define("env", "stake", stake_func).unwrap();

    let unstake_func = Func::wrap(&mut store, host::unstake);
    linker.define("env", "unstake", unstake_func).unwrap();

    let debug_func = Func::wrap(&mut store, host::debug);
    linker.define("env", "debug", debug_func).unwrap();

    // Instantiate the module
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| format!("Failed to instantiate module: {:?}", e))?
        .start(&mut store)
        .map_err(|e| format!("Failed to start instance: {:?}", e))?;

    // Call the execute function, passing the blockchain pointer
    // Split the pointer into two i32s
    let blockchain_ptr = blockchain as *mut Blockchain as usize; // Use usize to hold the full pointer
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
