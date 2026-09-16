pub type Slot = u64;
pub type Epoch = u64;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct SlotId {
    epoch: Epoch,
    slot: Slot,
}

impl SlotId {
    pub fn from_global_slot(global_slot: Slot, slots_per_epoch: Slot) -> Self {
        assert!(slots_per_epoch > 0);

        Self {
            epoch: global_slot / slots_per_epoch,
            slot: global_slot % slots_per_epoch,
        }
    }

    pub fn next(self, slots_per_epoch: Slot) -> Self {
        if self.slot + 1 >= slots_per_epoch {
            Self {
                epoch: self.epoch + 1,
                slot: 0,
            }
        } else {
            Self {
                epoch: self.epoch,
                slot: self.slot + 1,
            }
        }
    }

    pub fn epoch(&self) -> Epoch {
        self.epoch
    }

    pub fn slot(&self) -> Slot {
        self.slot
    }
}
