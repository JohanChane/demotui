use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};

use crate::tui::TuiWidget;

use super::dev::*;

#[macro_use]
mod profile;
mod template;

#[derive(Default)]
pub struct FileTab(DualTab<profile::Profile, template::Template>);

impl FileTab {
    pub const TITLES: [&str; 2] = [profile::Profile::TITLE, template::Template::TITLE];

    pub fn focused_on_profile(&mut self) {
        self.0.focused_on_c1();
    }
    pub fn focused_on_template(&mut self) {
        self.0.focused_on_c2();
    }
}

impl TuiWidget for FileTab {
    fn handle_key_event(&mut self, kv: &KeyEvent) {
        self.0.handle_key_event(kv);
    }

    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.0.render(f, area);
    }

    fn sync(&mut self) {
        self.0.sync();
    }
}
