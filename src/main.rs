fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use bchain::{
        primitives::{Transaction, TransactionType, Wallet},
        Blockchain,
    };

    #[test]
    fn test_wasm_simple_contract() {
        let mut blockchain = Blockchain::new();

        blockchain
            .wallets
            .wallets
            .insert("Alice".to_string(), Wallet::new(500.0));

        // Load the Wasm contract bytecode, that is very un-unittest like :D
        let wasm_bytes =
            std::fs::read("target/wasm32-unknown-unknown/release/counter_contract.wasm").unwrap();

        // Deploy the contract
        let tx1 = Transaction::new(
            "Alice".to_string(),
            TransactionType::DeployContract { code: wasm_bytes },
            1.0,
        );
        assert!(blockchain.add_block(vec![tx1]).is_ok());

        // Call the contract
        let tx2 = Transaction::new(
            "Alice".to_string(),
            TransactionType::CallContract {
                contract_address: "contract_0".to_string(),
            },
            1.0,
        );
        assert!(blockchain.add_block(vec![tx2]).is_ok());
    }

    #[test]
    fn test_contract_execution() {
        let mut blockchain = Blockchain::new();

        blockchain
            .wallets
            .wallets
            .insert("Alice".to_string(), Wallet::new(500.0));

        let wasm_bytes =
            std::fs::read("target/wasm32-unknown-unknown/release/counter_contract.wasm").unwrap();
        // Deploy the contract
        let tx1 = Transaction::new(
            "Alice".to_string(),
            TransactionType::DeployContract { code: wasm_bytes },
            1.0,
        );
        blockchain.add_block(vec![tx1]).unwrap();

        // Call the contract multiple times to increment the counter
        for i in 1..=5 {
            let tx = Transaction::new(
                "Alice".to_string(),
                TransactionType::CallContract {
                    contract_address: "contract_0".to_string(),
                },
                1.0,
            );
            blockchain.add_block(vec![tx]).unwrap();
            println!("After block {}: {:#?}", i, blockchain.wallets);
        }

        // Verify the results
        let alice_wallet = blockchain.wallets.wallets.get("Alice").unwrap();
        let bob_wallet = blockchain.wallets.wallets.get("Bob").unwrap();

        // After 5 calls:
        // - Counter should be 5
        // - Transfer: 50 tokens from Alice to Bob (happens once, since balance drops below 100 after the first call)
        // - Fees: 5 blocks * 1.0 = 5.0
        // - Staking: After counter > 2 (call 3), stake 10 tokens
        // - Unstaking: After counter > 4 (call 5), unstake 5 tokens
        assert_eq!(alice_wallet.balance, 435.0); // 500 - 50 (transfer) - 5 (fees) - 10 (stake) + 0 (unstake not yet processed)
        assert_eq!(alice_wallet.staked, 5.0); // 10 (stake) - 5 (unstake)
        assert!(!alice_wallet.pending_unstakes.is_empty());
        let alice_unstake = alice_wallet.pending_unstakes.front().unwrap();
        assert_eq!(alice_unstake.amount, 5.0);
        assert_eq!(alice_unstake.effective_epoch, 2); // Current epoch 5 + delay 2
        assert_eq!(bob_wallet.balance, 50.0);
    }
}
