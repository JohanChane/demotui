use ratatui::{
    prelude::{Buffer, Rect},
    widgets::Widget,
};

use crate::tui::frontend::FrontEnd;

pub(crate) struct Which<'a> {
    frontend: &'a FrontEnd,
}

impl<'a> Which<'a> {
    pub(crate) fn new(frontend: &'a FrontEnd) -> Self {
        Self { frontend }
    }
}

impl Widget for Which<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        //self.frontend.which
    }
}
