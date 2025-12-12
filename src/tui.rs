use crossterm::event::KeyEvent;
use utils::*;

mod app;
mod popmsg;
mod tab;
mod theme;
mod utils;
mod widget;

pub use app::App;
pub use theme::Theme;

trait TuiWidget {
    fn handle_key_event(&mut self, kv: &KeyEvent);
    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect);
    fn sync(&mut self);
}

pub fn init() -> anyhow::Result<()> {
    theme::Theme::load();
    raw_mode::setup()?;
    raw_mode::set_panic_hook();
    Ok(())
}

pub fn restore() -> anyhow::Result<()> {
    raw_mode::restore()?;
    Ok(())
}
