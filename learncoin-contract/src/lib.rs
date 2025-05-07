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
        fn store(
            blockchain_ptr_low: i32,
            blockchain_ptr_high: i32,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        ) -> i32;
        fn load(
            blockchain_ptr_low: i32,
            blockchain_ptr_high: i32,
            key_ptr: *const u8,
            key_len: u32,
            value_pte: i32,
        ) -> i32;
        fn stake(blockchain_ptr_low: i32, blockchain_ptr_high: i32, amount: f64) -> i32;
        fn unstake(blockchain_ptr_low: i32, blockchain_ptr_high: i32, amount: f64) -> i32;
        // Declare the debug function
        fn debug(msg_ptr: *const u8, msg_len: u32, value: u32);
    }
    // Helper function to log debug messages
    #[inline(always)]
    fn log_debug(msg: &str, value: u32) {
        unsafe {
            debug(msg.as_ptr(), msg.len() as u32, value);
        }
    }

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
    // We can't log balance directly as a u32 because it's an f64, so use 0 as a placeholder
    log_debug("Checked balance", 0);

    let key = "counter";
    let key_bytes = key.as_bytes();

    let mut counter_bytes = [0u8; 4];
    let value_ptr = counter_bytes.as_mut_ptr() as i32;

    let counter_len = unsafe {
        load(
            blockchain_ptr_low,
            blockchain_ptr_high,
            key_bytes.as_ptr(),
            key_bytes.len() as u32,
            value_ptr,
        )
    };
    let mut counter = if counter_len >= 0 {
        let counter_value = i32::from_le_bytes(counter_bytes);
        log_debug("Counter loaded", counter_value as u32);
        counter_value
    } else {
        log_debug("Counter not found", 0);
        0
    };

    counter += 1;
    log_debug("Counter incremented", counter as u32);

    let new_counter_bytes = counter.to_le_bytes();
    let result = unsafe {
        store(
            blockchain_ptr_low,
            blockchain_ptr_high,
            key_bytes.as_ptr(),
            key_bytes.len() as u32,
            new_counter_bytes.as_ptr(),
            new_counter_bytes.len() as u32,
        )
    };
    if result != 0 {
        log_debug("Store failed", 0);
        return 1;
    }
    log_debug("Counter stored", 0);

    if counter == 3 {
        log_debug("Staking 10 tokens", 0);
        let stake_result = unsafe { stake(blockchain_ptr_low, blockchain_ptr_high, 10.0) };
        if stake_result != 0 {
            log_debug("Stake failed", stake_result as u32);
            return 1;
        }
        log_debug("Stake succeeded", 0);
    }

    if counter > 4 {
        log_debug("Unstaking 5 tokens", 0);
        let unstake_result = unsafe { unstake(blockchain_ptr_low, blockchain_ptr_high, 5.0) };
        if unstake_result != 0 {
            log_debug("Unstake failed", unstake_result as u32);
            return 1;
        }
        log_debug("Unstake succeeded", 0);
    }

    // Check if the transfer has already happened
    let transferred_key = "transferred";
    let transferred_key_bytes = transferred_key.as_bytes();
    let mut transferred_bytes = [0u8; 1];
    let transferred_value_ptr = transferred_bytes.as_mut_ptr() as i32;

    let transferred_len = unsafe {
        load(
            blockchain_ptr_low,
            blockchain_ptr_high,
            transferred_key_bytes.as_ptr(),
            transferred_key_bytes.len() as u32,
            transferred_value_ptr,
        )
    };
    let has_transferred = if transferred_len >= 0 {
        transferred_bytes[0] == 1
    } else {
        false
    };

    if !has_transferred && balance > 101.0 {
        // Account for the 1.0 fee
        let from = "Alice";
        let to = "Bob";
        let amount = 50.0;
        let from_bytes = from.as_bytes();
        let to_bytes = to.as_bytes();
        log_debug("Transferring", 0);
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
            log_debug("Transfer succeeded", 0);
            // Mark the transfer as done
            transferred_bytes[0] = 1;
            let store_result = unsafe {
                store(
                    blockchain_ptr_low,
                    blockchain_ptr_high,
                    transferred_key_bytes.as_ptr(),
                    transferred_key_bytes.len() as u32,
                    transferred_bytes.as_ptr(),
                    transferred_bytes.len() as u32,
                )
            };
            if store_result != 0 {
                log_debug("Failed to store transferred flag", 0);
                return 1;
            }
            return 0;
        }
        log_debug("Transfer failed", result as u32);
    }

    log_debug("Execution completed", 0);
    1
}
