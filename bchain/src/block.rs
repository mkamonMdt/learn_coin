use sha2::{Digest, Sha256};

pub struct BlockMeta {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: String,
}

pub struct Block {
    pub meta: BlockMeta,
    pub data: Vec<u8>,
    pub nonce: u64,
    pub hash: String,
}

impl Block {
    pub fn new(meta: BlockMeta, data: Vec<u8>) -> Self {
        let mut block = Self {
            meta,
            data,
            nonce: 0,
            hash: "".to_string(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined with hash: {}", self.hash);
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let meta = &self.meta;
        hasher.update(format!(
            "{}{}{}{}",
            meta.index, meta.timestamp, meta.previous_hash, self.nonce
        ));
        hasher.update(&self.data);
        format!("{:x}", hasher.finalize())
    }
}
