use crate::bchain_error::BChainError;
use crate::primitives::Block;
use crate::primitives::Transaction;
use crate::primitives::Wallet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    ProduceBlock(String, Vec<Transaction>),
    IncommingBlock(Block),
    GetHeaders,
    Headers(Vec<Block>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub msg_type: MessageType,
}

pub trait BlockchainFacade {
    fn receive(&mut self, msg: Message) -> Result<(), BChainError>;
    fn get_wallet(&self, user: &str) -> Result<&Wallet, BChainError>;
}
