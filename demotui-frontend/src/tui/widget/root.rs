use demotui_shared::frontend;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::{
    core::popup::Popup,
    frontend::FrontEnd,
    widget::{confirm::Confirm, hello::Hello},
};

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

        // if (self.frontend.popup.visible) {
        //     Popup::new(self.frontend).render(area, buf);
        // }

        // if (self.frontend.confirm.visible) {
        //     Confirm::new(self.frontend).render(area, buf);
        // }

        // if (self.frontend.input.visible) {
        //     Input::new(self.frontend).render(area, buf);
        // }

        // if (check tab and self.frontend.prof_tmpl.visible) {
        //     render current main
        // }
    }
}
