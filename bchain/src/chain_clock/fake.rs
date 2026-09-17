use crate::chain_clock::ChainClock;
use crate::config::static_config::EPOCH_HEIGHT;
use crate::primitives::SlotId;

use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Notify;

#[derive(Clone)]
pub struct ChainClockFake {
    state: Arc<InnerChainClockFake>,
}

struct InnerChainClockFake {
    slot_id: Mutex<SlotId>,
    notify: Notify,
}

impl ChainClock for ChainClockFake {
    fn now(&self) -> SlotId {
        *self.state.slot_id.lock().unwrap()
    }

    async fn wait_next_slot(&self, last_seen: SlotId) -> SlotId {
        loop {
            let notified = self.state.notify.notified();
            let current = self.now();
            if current > last_seen {
                return current;
            }
            notified.await;
        }
    }
}

impl ChainClockFake {
    pub fn new() -> ChainClockFake {
        Self {
            state: Arc::new(InnerChainClockFake {
                slot_id: Mutex::new(SlotId::from_global_slot(0, EPOCH_HEIGHT)),
                notify: Notify::new(),
            }),
        }
    }

    fn wake(&self) {
        self.state.notify.notify_waiters();
    }

    pub fn set_next_slot(&mut self) {
        {
            let mut slot_id = self.state.slot_id.lock().unwrap();
            *slot_id = slot_id.next(EPOCH_HEIGHT)
        }
        self.wake();
    }

    pub fn adavance_slot(&mut self, n: u64) {
        {
            let mut slot_id = self.state.slot_id.lock().unwrap();
            let global_slot = slot_id.epoch() * EPOCH_HEIGHT + slot_id.slot() + n;
            *slot_id = SlotId::from_global_slot(global_slot, EPOCH_HEIGHT);
        }
        self.wake();
    }
}

#[tokio::test]
async fn waits_until_slot_advances() {
    let mut clock = ChainClockFake::new();
    let waiter_clock = clock.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

    let waiter = tokio::spawn(async move {
        let last_seen = SlotId::from_global_slot(0, EPOCH_HEIGHT);
        let _ = ready_tx.send(());

        waiter_clock.wait_next_slot(last_seen).await
    });

    ready_rx.await.unwrap();
    assert!(!waiter.is_finished());
    clock.set_next_slot();
    let result = waiter.await.unwrap();

    assert_eq!(result, SlotId::from_global_slot(1, EPOCH_HEIGHT));
}
