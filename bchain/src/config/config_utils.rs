use crate::static_config;

pub fn get_epoch(block_height: usize) -> usize {
    block_height / static_config::EPOCH_HEIGHT
}

pub fn get_validators_consensus_block(epoch: usize) -> usize {
    if epoch < 2 {
        0
    } else {
        (epoch - 1) * static_config::EPOCH_HEIGHT - 1
    }
}
