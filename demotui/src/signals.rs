use std::mem::swap;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use demotui_shared::frontend::event::FrontEndEvent;
use futures_lite::StreamExt;
use tokio::{select, task::spawn_blocking};

pub(super) struct Signals {}

impl Signals {
    pub(super) fn start() -> Result<Self> {
        Self::spawn();
        Ok(Self {})
    }

    fn handle_term(event: CrosstermEvent) {
        match event {
            CrosstermEvent::Key(
                key @ KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                },
            ) => FrontEndEvent::Key(key).emit(),
            CrosstermEvent::Resize(..) => FrontEndEvent::Resize.emit(),
            _ => {}
        }
    }

    pub fn spawn() -> Result<()> {
        let mut stream = Some(event::EventStream::new());
        tokio::spawn(async move {
            loop {
                if let Some(s) = &mut stream {
                    select! {
                        biased;
                        Some(Ok(e)) = s.next() => Self::handle_term(e),
                    }
                }
            }
        });

        Ok(())
    }
}
