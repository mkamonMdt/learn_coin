use crate::block::{Block, BlockMeta};
use sha2::{Digest, Sha256};

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
        }
    }

    fn add_block(&mut self, data: Vec<u8>) {
        let previous_block = &self.chain[self.chain.len() - 1];
        let meta = BlockMeta {
            index: previous_block.meta.index + 1,
            timestamp: 0,
            previous_hash: previous_block.hash.clone(),
        };

        let hash = calculate_hash(&meta, &data);

        self.chain.push(Block { meta, hash, data });
    }
}

fn calculate_hash(meta: &BlockMeta, data: &Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!(
        "{}{}{}",
        meta.index, meta.timestamp, meta.previous_hash
    ));
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
