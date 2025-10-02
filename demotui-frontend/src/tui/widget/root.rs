use demotui_shared::frontend;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::{frontend::FrontEnd, widget::hello::Hello};

pub struct Root<'a> {
    frontend: &'a FrontEnd,
}

impl<'a> Root<'a> {
    pub fn new(frontend: &'a FrontEnd) -> Self {
        Self { frontend }
    }
}

impl Widget for Root<'_> {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        Hello::new(self.frontend).render(area, buf);
    }
}
