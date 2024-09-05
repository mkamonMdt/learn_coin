use crate::{
    block::{Block, BlockMeta},
    constats::DIFFICULTY,
};

pub struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        Self {
            chain: vec![Self::genesis_block()],
        }
    }

    fn genesis_block() -> Block {
        Block {
            meta: BlockMeta {
                index: 0,
                timestamp: 0,
                previous_hash: String::from("0"),
            },
            hash: String::from("Genesis Hash"),
            data: String::from("Genesis Block").into(),
            nonce: 0,
        }
    }

    fn create_block(&mut self, data: Vec<u8>) {
        let previous_block = &self.chain[self.chain.len() - 1];
        let meta = BlockMeta {
            index: previous_block.meta.index + 1,
            timestamp: 0,
            previous_hash: previous_block.hash.clone(),
        };
        let mut block = Block::new(meta, data);
        block.mine(DIFFICULTY);

        self.chain.push(block);
    }

    fn add_block(&mut self, next: Block) {
        let current_top = &self.chain[self.chain.len() - 1];
        if validate_block(current_top, &next) {
            self.chain.push(next);
        }
    }
}

fn validate_block(current_reference: &Block, incoming_next: &Block) -> bool {
    if current_reference.meta.index + 1 != incoming_next.meta.index {
        return false;
    }
    if current_reference.meta.timestamp >= incoming_next.meta.timestamp {
        return false;
    }
    if current_reference.hash != incoming_next.meta.previous_hash {
        return false;
    }

    let target = "0".repeat(DIFFICULTY);
    if incoming_next.hash[..DIFFICULTY] != target {
        return false;
    }
    incoming_next.calculate_hash() != incoming_next.hash
}
