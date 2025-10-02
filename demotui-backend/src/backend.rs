use std::{
    cell::{Ref, RefCell},
    thread::spawn,
};

use anyhow::Result;
use demotui_shared::backend::event::BackEndEvent;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::dispatcher::{self, Dispatcher};
type FrontEndSenderType = Sender<std::io::Stdout>;

pub struct BackEnd {}

impl BackEnd {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn start() -> Result<()> {
        let mut backend = Self {};

        BackEnd::spawn(backend)?;

        Ok(())
    }

    fn spawn(mut self) -> Result<()> {
        let mut rx = BackEndEvent::take();
        let mut events = Vec::with_capacity(50);
        tokio::spawn(async move {
            let mut dispatcher = Dispatcher::new(&mut self);
            loop {
                match rx.recv_many(&mut events, 50).await {
                    0 => {
                        break;
                    }
                    n => {
                        for event in events.drain(..) {
                            if let Err(e) = dispatcher.dispatch(event) {
                                log::error!("Failed to handle backend event: {}", e);
                            }
                        }
                    }
                }
            }
        });
        log::debug!("backend exit");
        Ok(())
    }

    // pub fn handle_event(ev: BackEndEvent) {
    //     match ev {
    //         BackEndEvent::Call(data) => { /* do something */ }
    //     }
    // }
}
