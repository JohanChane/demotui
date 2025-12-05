use crate::tui::TuiWidget;
use crossterm::event::KeyEvent;
use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::Block;
use std::sync::{LazyLock, Mutex, mpsc};
use tokio::sync::oneshot::Sender;

mod builder;
mod confirm;
mod wrapper;

pub use builder::MsgBuilder;
pub use confirm::Confirm;
use wrapper::{Prompt, Wrapped};

static PAIR: LazyLock<Mutex<(mpsc::Sender<Wrapped>, mpsc::Receiver<Wrapped>)>> =
    LazyLock::new(|| Mutex::new(mpsc::channel()));

pub enum Route {
    Keep,
    Send,
    Drop,
}

pub trait Msg {
    type Result;

    fn match_key_event(&mut self, kv: &KeyEvent) -> Route;
    fn send(self, tx: Sender<Self::Result>);
    fn render(&self, f: &mut Frame, area: Rect, block: Block, is_focused: bool);
    /// (Width, Height)
    fn size(&self) -> (u16, u16);
}

#[derive(Default)]
pub struct PopUp {
    content: Vec<Wrapped>,
}

impl PopUp {
    pub fn check(&mut self) -> bool {
        !self.content.is_empty()
    }
}

impl TuiWidget for PopUp {
    fn handle_key_event(&mut self, kv: &KeyEvent) {
        if let Some(instance) = self.content.last_mut() {
            match instance.handle_key_event(kv) {
                Route::Keep => {}
                Route::Send => {
                    self.content.pop().unwrap().send();
                }
                Route::Drop => {
                    let _ = self.content.pop();
                }
            }
        }
    }

    fn render(&mut self, f: &mut Frame, _: Rect) {
        if let Some(cell) = self.content.last_mut() {
            cell.render(f);
        }
    }

    fn sync(&mut self) {
        while let Ok(content) = PAIR.lock().unwrap().1.try_recv() {
            self.content.push(content.into());
        }
    }
}

struct Instance<C: Msg<Result = R>, R> {
    content: C,

    title: String,
    prompt: Option<Prompt>,
    is_focus_prompt: bool,

    tx: Sender<R>,
}
