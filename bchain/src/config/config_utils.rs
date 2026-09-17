use crate::config::static_config;
use crate::primitives::Epoch;
use crate::primitives::Slot;

pub fn get_epoch(block_height: Slot) -> Epoch {
    (block_height / static_config::EPOCH_HEIGHT) as Epoch
}

pub fn get_validators_consensus_block(epoch: Epoch) -> Slot {
    if epoch < 2 {
        0
    } else {
        (epoch - 1) * static_config::EPOCH_HEIGHT - 1
    }
}
