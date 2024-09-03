pub struct BlockMeta {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: String,
}

pub struct Block {
    pub meta: BlockMeta,
    pub hash: String,
    pub data: Vec<u8>,
}
