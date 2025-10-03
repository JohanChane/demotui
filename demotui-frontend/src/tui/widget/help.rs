use ratatui::{
    prelude::{Buffer, Rect},
    widgets::Widget,
};

use crate::tui::frontend::FrontEnd;

pub(crate) struct Help<'a> {
    frontend: &'a FrontEnd,
}

impl<'a> Help<'a> {
    pub(crate) fn new(frontend: &'a FrontEnd) -> Self {
        Self { frontend }
    }
}

impl Widget for Help<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        //self.frontend.help
    }
}
