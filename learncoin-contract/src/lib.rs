#[no_mangle]
pub extern "C" fn execute(blockchain_ptr_low: i32, blockchain_ptr_high: i32) -> i32 {
    // External functions provided by LearnCoin (to be defined)
    extern "C" {
        fn get_balance(
            blockchain_ptr_low: i32,
            blockchain_ptr_high: i32,
            user_ptr: *const u8,
            user_len: u32,
        ) -> f64;
        fn transfer(
            blockchain_ptr_low: i32,
            blockchain_ptr_high: i32,
            from_ptr: *const u8,
            from_len: u32,
            to_ptr: *const u8,
            to_len: u32,
            amount: f64,
        ) -> i32;
    }

    // Hardcoded user data (in a real contract will be passed as arguments)
    let user = "Alice";
    let user_bytes = user.as_bytes();
    let balance = unsafe {
        get_balance(
            blockchain_ptr_low,
            blockchain_ptr_high,
            user_bytes.as_ptr(),
            user_bytes.len() as u32,
        )
    };

    // If Alice's balance > 100, transfer 50 to Bob
    if balance > 100.0 {
        let from = "Alice";
        let to = "Bob";
        let amount = 50.0;
        let from_bytes = from.as_bytes();
        let to_bytes = to.as_bytes();
        let result = unsafe {
            transfer(
                blockchain_ptr_low,
                blockchain_ptr_high,
                from_bytes.as_ptr(),
                from_bytes.len() as u32,
                to_bytes.as_ptr(),
                to_bytes.len() as u32,
                amount,
            )
        };
        if result == 0 {
            return 0; // Success
        }
    }
    1 //Failure
}
