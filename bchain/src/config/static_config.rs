use crate::primitives::Slot;
use std::time::Duration;

pub const EPOCH_HEIGHT: Slot = 10;
pub const BLOCK_CHAIN_WORTH: f64 = 1000.0;
pub const GENESIS: &str = "Genesis";
pub const REWARD_RATE_PER_EPOCH: f64 = 0.00001;
pub const SLOT_DURATION: Duration = Duration::from_secs(1);
