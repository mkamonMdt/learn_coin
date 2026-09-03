use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::bchain_error::BChainError;
use crate::bchain_error::BChainResult;
use crate::primitives::Transaction;
use crate::primitives::Wallet;

pub enum BlockchainCommand {
    ProduceBlock {
        validator: String,
        transactions: Vec<Transaction>,
    },

    GetWallet {
        user: String,
        response: oneshot::Sender<BChainResult<Wallet>>,
    },
}

#[derive(Clone)]
pub struct CommandHandle {
    pub(super) tx: mpsc::Sender<BlockchainCommand>,
}

impl CommandHandle {
    pub async fn produce_block(
        &self,
        validator: String,
        transactions: Vec<Transaction>,
    ) -> BChainResult<()> {
        self.tx
            .send(BlockchainCommand::ProduceBlock {
                validator,
                transactions,
            })
            .await
            .map_err(|_| BChainError::DummyError("TODO".to_string()))
    }

    pub async fn get_wallet(
        &self,
        user: String,
        response: oneshot::Sender<BChainResult<Wallet>>,
    ) -> BChainResult<()> {
        self.tx
            .send(BlockchainCommand::GetWallet { user, response })
            .await
            .map_err(|_| BChainError::DummyError("TODO".to_string()))
    }
}
