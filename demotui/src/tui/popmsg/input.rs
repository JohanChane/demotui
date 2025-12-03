use super::dev::*;

#[derive(Default)]
pub struct Input {
    buffer: String,
    cursor: usize,
}

impl Msg for Input {
    type Result = String;

    fn match_key_event(&mut self, kv: &KeyEvent) -> Route {
        match kv.code {
            KeyCode::Enter => return Route::Send,
            KeyCode::Esc => return Route::Drop,
            KeyCode::Char(ch) => {}
            KeyCode::Backspace => {}
            KeyCode::Left => {}
            KeyCode::Right => {}
            _ => {}
        }
        Route::Keep
    }

    fn send(self, tx: Sender<Self::Result>) {
        tx.send(self.buffer).unwrap()
    }

    fn render(&self, f: &mut Frame, area: Rect, block: Block) {
        todo!()
    }

    fn size(&self) -> (u16, u16) {
        (todo!(), 1)
    }
}

impl Input {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_title(self, title: String) -> MsgBuilder<Self> {
        MsgBuilder::new(self, title)
    }
}
