use std::time::Duration;

use crossbeam::{
    channel::{Receiver, Sender, bounded},
    sync::{Parker, Unparker},
};

#[derive(Debug)]
pub enum WakeReason {
    Timeout,
    Aborted,
}

pub struct AbortableSleep(Parker, Receiver<WakeReason>);

impl AbortableSleep {
    pub fn new() -> (Self, AbortToken) {
        let (tx, rx) = bounded(1);

        let p = Parker::new();
        let unparker = p.unparker().clone();
        (Self(p, rx), AbortToken(unparker, tx))
    }

    pub fn sleep(&self, timeout: Duration) -> WakeReason {
        self.0.park_timeout(timeout);
        self.1.try_recv().unwrap_or(WakeReason::Timeout)
    }
}

#[derive(Clone, Debug)]
pub struct AbortToken(Unparker, Sender<WakeReason>);

impl AbortToken {
    pub fn abort(&self) {
        // errors if receiver half was dropped; if it was dropped, then
        // whoever it is is no longer using this type
        if self.1.try_send(WakeReason::Aborted).is_ok() {
            self.0.unpark();
        }
    }
}
