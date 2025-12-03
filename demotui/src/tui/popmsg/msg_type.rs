use super::context::WidgetState;
use super::*;
use crossterm::event::KeyCode;
use ratatui::widgets::{Paragraph, Wrap};
use tokio::sync::oneshot;

const WRAP_TRUE: Wrap = Wrap { trim: true };


pub struct Input {
    title: String,
    prompt: Option<String>,

    tx: oneshot::Sender<String>,
}

impl PopMsg for Input {
    fn match_key_event(&mut self, kv: &KeyEvent, ctx: &mut Context) -> Route {
        match kv.code {
            KeyCode::Enter => return Route::Send,
            KeyCode::Esc => return Route::Drop,
            _ => ctx.handle_key_event(kv),
        }
        Route::Keep
    }
    fn send(self: Box<Self>, ctx: Context) {
        if let WidgetState::Buffer { buffer, .. } = ctx.widget {
            self.tx.send(buffer).unwrap()
        }
    }
    fn config(&self) -> Context {
        Context::buffer().with_prompt()
    }
    fn render(&self, f: &mut ratatui::Frame, ctx: &mut Context) {
        const SIZE: (u16, u16) = (20, 1);

        if let Some(prompt) = self.prompt.as_deref() {
            todo!()
        } else {
            ctx.widget.as_widget(); // 20 1
        }
        todo!()
    }
}

fn centered_area((width, height): (u16, u16), f: &ratatui::Frame) -> ratatui::layout::Rect {
    todo!()
}
