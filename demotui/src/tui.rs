use crossterm::event::KeyEvent;
use tab::prelude::*;
use widget::popmsg::PopUp;

mod popmsg;
mod tab;
mod widget;

trait TuiWidget {
    fn handle_key_event(&mut self, kv: &KeyEvent);
    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect);
}

// 50fps
const TICK_RATE: std::time::Duration = std::time::Duration::from_millis(20);

pub struct App {
    tab1: FileTab,
    popup: PopUp,

    tab_index: u8,
    should_quit: bool,
}
impl App {
    pub fn new() -> Self {
        Self {
            tab1: FileTab::default(),
            popup: PopUp::default(),
            tab_index: 0,
            should_quit: false,
        }
    }
    fn handle_key_event(&mut self, kv: &KeyEvent) {
        if self.popup.check() {
            self.popup.handle_key_event(kv);
        } else {
            // Todo: 匹配0..=9和Tab 以在Tab间移动
            match self.tab_index {
                0..=1 => self.tab1.handle_key_event(kv),
                2.. => unreachable!(),
            }
        }
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        // Todo:  将原本的tabbar和statusbar移过来
        //以及 区块划分 部分代码

        match self.tab_index {
            // Notice: FileTab是特殊的双Tab
            0..=1 => self.tab1.render(f, todo!()),
            2.. => unreachable!(),
        }

        if self.popup.check() {
            self.popup.render(f, Default::default());
        }
    }

    #[tokio::main]
    pub async fn serve(mut self) -> anyhow::Result<()> {
        let mut events = crossterm::event::EventStream::new();
        let mut invt = tokio::time::interval(TICK_RATE);
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;

        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;

            let ev = {
                use futures_lite::StreamExt as _;
                let mut tick = Box::pin(invt.tick());
                let ev = tokio::select! {
                    Some(ev) = events.next() => ev?,
                    _ = &mut tick => continue,
                };
                tick.await;
                ev
            };

            use crossterm::event::Event;
            match ev {
                Event::Key(key_event) => {
                    // #[cfg(debug_assertions)]
                    // the_egg(key_event.code);
                    self.handle_key_event(&key_event);
                }
                Event::Resize(..) => terminal.autoresize()?,
                _ => (),
            }
        }

        log::trace!("App Exit");
        Ok(())
    }
}
