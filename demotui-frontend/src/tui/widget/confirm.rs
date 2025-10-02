use ratatui::{
    prelude::{Buffer, Rect},
    widgets::Widget,
};

use crate::tui::frontend::FrontEnd;

pub(crate) struct Confirm<'a> {
    frontend: &'a FrontEnd,
}

impl<'a> Confirm<'a> {
    pub(crate) fn new(frontend: &'a FrontEnd) -> Self {
        Self { frontend }
    }
}

impl Widget for Confirm<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        //self.frontend.confirm
    }
}
