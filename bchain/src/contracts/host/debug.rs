use wasmi::{Caller, Extern};

pub fn debug(caller: Caller<(String, String)>, msg_ptr: i32, msg_len: i32, value: u32) {
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
