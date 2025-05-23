use crate::bchain_error::BChainError;
use crate::primitives::block::Block;
use crate::primitives::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Transaction(Transaction),
    Block(Block),
    GetHeaders,
    Headers(Vec<Block>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub msg_type: MessageType,
}

#[async_trait::async_trait]
pub trait MessageHandler {
    async fn receive(&mut self, msg: &Message) -> Result<Message, BChainError>;
}
