use bchain::primitives::block::Block;
use bchain::primitives::transaction::Transaction;
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
