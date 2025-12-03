use crate::tui::TuiWidget;
use crossterm::event::KeyEvent;
use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::{Block, Clear, Paragraph};
use std::sync::mpsc;
use std::sync::{LazyLock, Mutex};
use tokio::sync::oneshot::{Receiver, Sender, channel};

type Wrapped = Box<dyn Wrapper + Send>;

static PAIR: LazyLock<Mutex<(mpsc::Sender<Wrapped>, mpsc::Receiver<Wrapped>)>> =
    LazyLock::new(|| Mutex::new(mpsc::channel()));

#[derive(Default)]
pub struct PopUp {
    content: Vec<Wrapped>,
}

impl PopUp {
    pub fn check(&mut self) -> bool {
        if let Ok(content) = PAIR.lock().unwrap().1.try_recv() {
            self.content.push(content.into());
        }
        return !self.content.is_empty();
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
}

pub enum Route {
    Keep,
    Send,
    Drop,
}

pub trait Msg {
    type Result;

    fn match_key_event(&mut self, kv: &KeyEvent) -> Route;
    fn send(self, tx: Sender<Self::Result>);
    fn render(&self, f: &mut Frame, area: Rect, block: Block);
    fn size(&self) -> (u16, u16);
}

pub struct MsgBuilder<C> {
    content: C,

    title: String,
    prompt: Option<String>,
}
impl<C: Msg<Result = R> + Send + 'static, R: Send + 'static> MsgBuilder<C> {
    pub fn new(content: C, title: String) -> Self {
        Self {
            content,
            title,
            prompt: None,
        }
    }
    pub fn with_prompt(self, prompt: String) -> Self {
        Self {
            prompt: Some(prompt),
            ..self
        }
    }
    pub fn build_and_send(self) -> Receiver<R> {
        let (tx, rx) = channel();

        let Self {
            content,
            title,
            prompt,
        } = self;

        let cell = Instance {
            content,
            title,
            prompt,
            tx,
            vect_offset: 0,
            hori_offset: 0,
            page_size: 0,
            is_focus_prompt: false,
        };

        PAIR.lock().unwrap().0.send(Box::new(cell)).unwrap();

        rx
    }
}

trait Wrapper {
    fn handle_key_event(&mut self, kv: &KeyEvent) -> Route;
    fn render(&mut self, f: &mut Frame);
    fn send(self: Box<Self>);
}

struct Instance<C: Msg<Result = R>, R> {
    content: C,

    title: String,
    prompt: Option<String>,
    vect_offset: u8,
    hori_offset: u8,
    page_size: u8,
    is_focus_prompt: bool,

    tx: Sender<R>,
}

impl<C: Msg<Result = R>, R> Wrapper for Instance<C, R> {
    fn handle_key_event(&mut self, kv: &KeyEvent) -> Route {
        if self.prompt.is_some() && self.is_focus_prompt {
            Route::Keep
        } else {
            self.content.match_key_event(kv)
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let area = {
            let size = self.content.size();
            let area = f.area();
            todo!()
        };
        f.render_widget(Clear, area);
        if let Some(prompt) = self.prompt.as_deref() {
            todo!()
        } else {
            self.content
                .render(f, area, Block::bordered().title(self.title));
        }
    }

    fn send(self: Box<Self>) {
        self.content.send(self.tx);
    }
}
