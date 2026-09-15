use crate::primitives::SlotId;

pub trait ChainClock {
    fn now(&self) -> SlotId;

    async fn wait_next_slot(&self, last_seen: SlotId) -> SlotId;
}
