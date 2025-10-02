use demotui_shared::frontend;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::tui::frontend::FrontEnd;

pub(crate) struct Hello<'a> {
    frontend: &'a FrontEnd,
}

impl<'a> Hello<'a> {
    pub(crate) fn new(frontend: &'a FrontEnd) -> Self {
        Self { frontend }
    }
}

impl Widget for Hello<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 创建文本内容
        let hello_text = Line::from(vec![
            Span::styled(
                "Hello",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "World!",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        // 计算居中位置
        let x = area.x + (area.width.saturating_sub(hello_text.width() as u16)) / 2;
        let y = area.y + area.height / 2;

        // 在缓冲区中渲染文本
        buf.set_line(x, y, &hello_text, area.width);
    }
}
