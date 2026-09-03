use clap::Parser;
use cli::run_cli;
use client::client_control::CtrlCommand;
use client::run_clinet;
use tokio::sync::mpsc;

mod cli;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "40001")]
    listen_port: u16,

    #[arg(long)]
    connect_to: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let listen_addr = format!("127.0.0.1:{}", args.listen_port);

    let (tx, rx) = mpsc::channel::<CtrlCommand>(10);
    tokio::spawn(async move {
        run_clinet(listen_addr, rx).await;
    });

    if let Some(peer_addr) = args.connect_to {
        let _ = tx.send(CtrlCommand::InitiateConnection(peer_addr)).await;
    }

    run_cli(tx);
}

#[cfg(test)]
mod tests {
    use bchain::{
        primitives::{Transaction, TransactionType, Wallet},
        service::{BlockchainService, CommandHandle},
        GENESIS,
    };
    use tokio::sync::oneshot;

    async fn produce_block_with_single_tx(blockchain_handle: &CommandHandle, tx: Transaction) {
        let res = blockchain_handle
            .produce_block(GENESIS.to_owned(), vec![tx])
            .await;
        assert!(res.is_ok());
    }

    async fn insert_wallet(blockchain_handle: &CommandHandle, user: &str, amount: f64) {
        let tx = Transaction::new(
            GENESIS.to_owned(),
            TransactionType::Transfer {
                sender: GENESIS.to_owned(),
                receiver: user.to_owned(),
                amount,
            },
            0.,
        );
        produce_block_with_single_tx(blockchain_handle, tx).await;
    }

    async fn get_wallet(blockchain_handle: &CommandHandle, user: &str) -> Wallet {
        let (tx, rx) = oneshot::channel();
        let resp = blockchain_handle.get_wallet(user.to_string(), tx).await;

        assert!(resp.is_ok());
        let resp = rx.await;
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert!(resp.is_ok());
        resp.unwrap()
    }

    #[tokio::test]
    async fn test_wasm_simple_contract() {
        let blockchain = BlockchainService::start();
        let handle = blockchain.get_cmd_handle();

        insert_wallet(&handle, "Alice", 500.0).await;

        // Load the Wasm contract bytecode, that is very un-unittest like :D
        let wasm_bytes =
            std::fs::read("target/wasm32-unknown-unknown/release/counter_contract.wasm").unwrap();

        // Deploy the contract
        let tx1 = Transaction::new(
            "Alice".to_string(),
            TransactionType::DeployContract { code: wasm_bytes },
            1.0,
        );
        produce_block_with_single_tx(&handle, tx1).await;

        // Call the contract
        let tx2 = Transaction::new(
            "Alice".to_string(),
            TransactionType::CallContract {
                contract_address: "contract_0".to_string(),
            },
            1.0,
        );
        produce_block_with_single_tx(&handle, tx2).await;
    }

    #[tokio::test]
    async fn test_contract_execution() {
        let blockchain = BlockchainService::start();
        let handle = blockchain.get_cmd_handle();

        insert_wallet(&handle, "Alice", 500.0).await;

        let wasm_bytes =
            std::fs::read("target/wasm32-unknown-unknown/release/counter_contract.wasm").unwrap();
        // Deploy the contract
        let tx1 = Transaction::new(
            "Alice".to_string(),
            TransactionType::DeployContract { code: wasm_bytes },
            1.0,
        );
        produce_block_with_single_tx(&handle, tx1).await;

        // Call the contract multiple times to increment the counter
        for _ in 1..=5 {
            let tx = Transaction::new(
                "Alice".to_string(),
                TransactionType::CallContract {
                    contract_address: "contract_0".to_string(),
                },
                1.0,
            );
            produce_block_with_single_tx(&handle, tx).await;
            /*
            let alice_wallet = get_wallet(&handle, "Alice").await;
            let bob_wallet = get_wallet(&handle, "Bob").await;
            println!(
                "After block {}:\nAlice={:#?}\nBob={:#?}",
                i, alice_wallet, bob_wallet
            );
            */
        }

        // Verify the results

        let alice_wallet = get_wallet(&handle, "Alice").await;
        let bob_wallet = get_wallet(&handle, "Bob").await;

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
