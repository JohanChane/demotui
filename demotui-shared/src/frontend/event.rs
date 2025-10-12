use super::op::FrontEndOp;
use crossterm::event::KeyEvent;

use crate::ro_cell::RoCell;
use tokio::sync::mpsc;

pub static NEED_RENDER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static TX: RoCell<mpsc::UnboundedSender<FrontEndEvent>> = RoCell::new();
static RX: RoCell<mpsc::UnboundedReceiver<FrontEndEvent>> = RoCell::new();

pub enum FrontEndEvent {
    Call(FrontEndOp),
    Key(KeyEvent),
    Seq(Vec<FrontEndOp>),
    Render,
    Resize,
    Quit(EventQuit), // support Ctrl+C to exit app
}

#[derive(Debug, Default)]
pub struct EventQuit {
    pub code: i32,
}

impl FrontEndEvent {
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
