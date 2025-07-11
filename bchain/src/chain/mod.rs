use crate::config::static_config;
use crate::primitives::block::Block;
use crate::primitives::transaction::{Transaction, TransactionType};

#[derive(Debug)]
pub struct Chain {
    chain: Vec<Block>,
}

impl Chain {
    pub fn new(state_root: String) -> Self {
        let genesis_block = Block::new(
            vec![Transaction::new(
                static_config::GENESIS.to_string(),
                TransactionType::Transfer {
                    sender: static_config::GENESIS.to_string(),
                    receiver: "System".to_string(),
                    amount: static_config::BLOCK_CHAIN_WORTH,
                },
                0.0,
            )],
            "0".to_string(),
            static_config::GENESIS.to_string(),
            state_root,
        );
        Self {
            chain: vec![genesis_block],
        }
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn get_block_by_idx(&self, idx: usize) -> Option<&Block> {
        self.chain.get(idx)
    }

    pub fn get_last_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn push(&mut self, block: Block) {
        self.chain.push(block);
    }
}
