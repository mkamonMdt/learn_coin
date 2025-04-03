fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use bchain::{
        primitives::transaction::{Transaction, TransactionType},
        Blockchain, Wallet,
    };

    #[test]
    fn test_wasm_simple_contract() {
        let mut blockchain = Blockchain::new();

        blockchain
            .wallets
            .insert("Alice".to_string(), Wallet::new(500.0));

        // Load the Wasm contract bytecode, that is very un-unittest like :D
        let wasm_bytes =
            std::fs::read("target/wasm32-unknown-unknown/release/learncoin_contract.wasm").unwrap();

        // Deploy the contract
        let tx1 = Transaction::new(TransactionType::DeployContract { code: wasm_bytes }, 1.0);
        assert!(blockchain.add_block(vec![tx1]).is_ok());

        // Call the contract
        let tx2 = Transaction::new(
            TransactionType::CallContract {
                contract_address: "contract_0".to_string(),
            },
            1.0,
        );
        assert!(blockchain.add_block(vec![tx2]).is_ok());
    }
}
