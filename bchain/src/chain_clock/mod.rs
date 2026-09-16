use tokio::time::sleep_until;
use tokio::time::Instant;

use crate::config::static_config::EPOCH_HEIGHT;
use crate::config::static_config::SLOT_DURATION;
use crate::primitives::Slot;
use crate::primitives::SlotId;

use std::time::Duration;
use std::time::SystemTime;

pub trait ChainClock {
    fn now(&self) -> SlotId;

    async fn wait_next_slot(&self, last_seen: SlotId) -> SlotId;
}

pub struct SimpleClock {
    genesis_time: SystemTime,
}

impl ChainClock for SimpleClock {
    fn now(&self) -> SlotId {
        let elapsed = self.genesis_time.elapsed().unwrap_or_else(|e| {
            println!("SimpleClock failure: {e}");
            Duration::default()
        });

        let global_slot = (elapsed.as_nanos() / SLOT_DURATION.as_nanos()) as Slot;
        SlotId::from_global_slot(global_slot, EPOCH_HEIGHT)
    }

    async fn wait_next_slot(&self, last_seen: SlotId) -> SlotId {
        let next_slot = last_seen.next(EPOCH_HEIGHT);
        let next_slot_start = self.slot_start(next_slot);

        let now = SystemTime::now();

        if next_slot_start > now {
            let delay = next_slot_start
                .duration_since(now)
                .expect("time ordering changed");

            sleep_until(Instant::now() + delay).await;
        }

        // Recalculate after sleeping. This handles delays and missed slots.
        let current = self.now();

        if current >= next_slot {
            current
        } else {
            next_slot
        }
    }
}
impl SimpleClock {
    fn slot_start(&self, id: SlotId) -> SystemTime {
        let global_slot = id
            .epoch()
            .checked_mul(EPOCH_HEIGHT)
            .and_then(|v| v.checked_add(id.slot()))
            .expect("slot number overflow");

        let multiplier = u32::try_from(global_slot).expect("duration multiplier overflow");

        self.genesis_time
            + SLOT_DURATION
                .checked_mul(multiplier)
                .expect("slot duration overflow")
    }
}
