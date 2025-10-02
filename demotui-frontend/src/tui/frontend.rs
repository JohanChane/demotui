use anyhow::Result;

use tokio::sync::mpsc::{Receiver, Sender};

use demotui_shared::frontend::event::FrontEndEvent;

pub struct FrontEnd {}

impl FrontEnd {
    pub fn new() -> Self {
        Self {}
    }
}
