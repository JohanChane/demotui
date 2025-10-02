use super::op::BackEndOp;
use crossterm::event::KeyEvent;

use crate::ro_cell::RoCell;
use tokio::sync::mpsc;

static TX: RoCell<mpsc::UnboundedSender<BackEndEvent>> = RoCell::new();
static RX: RoCell<mpsc::UnboundedReceiver<BackEndEvent>> = RoCell::new();

pub enum BackEndEvent {
    Call(BackEndOp),
}

impl BackEndEvent {
    #[inline]
    pub fn init() {
        let (tx, rx) = mpsc::unbounded_channel();
        TX.init(tx);
        RX.init(rx);
    }

    #[inline]
    pub fn take() -> mpsc::UnboundedReceiver<Self> {
        RX.drop()
    }

    #[inline]
    pub fn emit(self) {
        TX.send(self).ok();
    }
}
