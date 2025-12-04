use crossterm::event::KeyEvent;
use tab::prelude::*;
use widget::popmsg::PopUp;

mod popmsg;
mod tab;
mod theme;
mod widget;

trait TuiWidget {
    fn handle_key_event(&mut self, kv: &KeyEvent);
    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect);
}

// 50fps
const TICK_RATE: std::time::Duration = std::time::Duration::from_millis(20);

pub struct App {
    file_tab: FileTab,
    popup: PopUp,

    tab_index: u8,
    should_quit: bool,
}
impl App {
    pub fn new() -> Self {
        Self {
            file_tab: FileTab::default(),
            popup: PopUp::default(),
            tab_index: 0,
            should_quit: false,
        }
    }
    fn handle_global_kv(&mut self, kv: &KeyEvent) -> bool {
        if matches!(kv.kind, crossterm::event::KeyEventKind::Press) {
            use crossterm::event::KeyCode;
            const TAB_COUNT: u8 = 2;
            match kv.code {
                KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                    if let Some(idx) = c.to_digit(10) {
                        self.tab_index = (idx as u8).min(TAB_COUNT) - 1
                    }
                }
                KeyCode::Tab => {
                    if self.tab_index == TAB_COUNT - 1 {
                        self.tab_index = 0
                    } else {
                        self.tab_index += 1
                    }
                }
                _ => return false,
            }
            return true;
        }
        false
    }
    fn handle_key_event(&mut self, kv: &KeyEvent) {
        if self.popup.check() {
            self.popup.handle_key_event(kv);
        } else if !self.handle_global_kv(kv) {
            match self.tab_index {
                0..=1 => self.file_tab.handle_key_event(kv),
                2.. => unreachable!(),
            }
        }
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        use ratatui::prelude::{Constraint, Layout};
        // Todo:  将原本的tabbar和statusbar移过来

        // split terminal into three part
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .split(f.area());

        render_tabbar(FileTab::TITLES.into_iter(), self.tab_index, f, chunks[0]);

        match self.tab_index {
            n @ 0..=1 => {
                self.file_tab.is_focus_profile = n == 0;
                self.file_tab.render(f, chunks[1])
            }
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
                    #[cfg(debug_assertions)]
                    the_egg(key_event.code);
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

pub(super) fn render_tabbar(
    titles: impl Iterator<Item = &'static str>,
    selected: u8,
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
) {
    use crate::tui::theme::Theme;
    use ratatui::style::Styled;
    use ratatui::widgets::{Block, Tabs};

    let titles = titles.map(|s| s.set_style(Theme::get().bars.tabbar_text));
    let this = Tabs::new(titles)
        .block(Block::bordered())
        .highlight_style(Theme::get().bars.tabbar_highlight)
        .select(Some(selected as usize));
    f.render_widget(this, area);
}

#[cfg(debug_assertions)]
fn the_egg(key: crossterm::event::KeyCode) {
    use crossterm::event::KeyCode;
    static INSTANCE: std::sync::Mutex<u8> = std::sync::Mutex::new(0);
    let mut current = INSTANCE.lock().unwrap();
    match *current {
        0 | 1 if matches!(key, KeyCode::Up) => (),
        2 | 3 if matches!(key, KeyCode::Down) => (),
        4 | 6 if matches!(key, KeyCode::Left) => (),
        5 | 7 if matches!(key, KeyCode::Right) => (),
        8 | 10 if matches!(key, KeyCode::Char('b') | KeyCode::Char('B')) => (),
        9 | 11 if matches!(key, KeyCode::Char('a') | KeyCode::Char('A')) => (),
        _ => {
            *current = 0;
            return;
        }
    }
    *current += 1;
    if *current == 12 {
        log::debug!("You've found the egg!")
    }
}

pub fn init() -> anyhow::Result<()> {
    theme::Theme::load();
    Ok(())
}
