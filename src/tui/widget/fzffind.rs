use std::cell::Cell;

use crate::tui::widget::popmsg::{Msg, Route};
use crate::tui::Key;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::Frame,
    style::{Style, Stylize as _},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use tokio::sync::oneshot::Sender;
use unicode_width::UnicodeWidthStr;

pub trait FzfItem {
    fn display(&self) -> &str;

    fn fuzzy_match(&self, query: &str) -> Option<Vec<usize>> {
        if query.is_empty() {
            return Some(Vec::new());
        }
        let lower_display = self.display().to_lowercase();
        let lower_query = query.to_lowercase();
        let mut pos = 0usize;
        let mut positions = Vec::with_capacity(query.len());
        for qc in lower_query.chars() {
            let found = lower_display[pos..].find(qc)?;
            pos += found + qc.len_utf8();
            positions.push(pos - qc.len_utf8());
        }
        Some(positions)
    }
}

impl FzfItem for String {
    fn display(&self) -> &str {
        self.as_str()
    }
}

impl FzfItem for &str {
    fn display(&self) -> &str {
        self
    }
}

pub struct FzfFind<I: FzfItem> {
    items: Vec<I>,
    filtered: Vec<usize>,
    query: String,
    selected: Cell<usize>,
}

impl<I: FzfItem + Send + 'static> FzfFind<I> {
    pub fn new(items: Vec<I>) -> Self {
        let filtered: Vec<usize> = (0..items.len()).collect();
        Self {
            items,
            filtered,
            query: String::new(),
            selected: Cell::new(0),
        }
    }

    pub fn with_title(self, title: String) -> crate::tui::widget::popmsg::MsgBuilder<Self> {
        crate::tui::widget::popmsg::MsgBuilder::new(self, title)
    }

    fn recompute_filter(&mut self) {
        self.filtered = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| item.fuzzy_match(&self.query).map(|_| i))
            .collect();
        if self.selected.get() >= self.filtered.len() && !self.filtered.is_empty() {
            self.selected.set(self.filtered.len() - 1);
        } else if self.filtered.is_empty() {
            self.selected.set(0);
        }
    }
}

impl<I: FzfItem + Send + 'static> Msg for FzfFind<I> {
    type Result = Option<usize>;

    fn match_key_event(&mut self, kv: &Key) -> Route {
        match kv.code {
            KeyCode::Esc => {
                return Route::Drop;
            }
            KeyCode::Enter => {
                if self.filtered.is_empty() {
                    return Route::Keep;
                }
                return Route::Send;
            }
            KeyCode::Char(c) if kv.ctrl && c == 'u' => {
                self.query.clear();
                self.recompute_filter();
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.recompute_filter();
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.recompute_filter();
            }
            KeyCode::Up => {
                let sel = self.selected.get();
                if sel > 0 {
                    self.selected.set(sel - 1);
                }
            }
            KeyCode::Down => {
                let sel = self.selected.get();
                if sel + 1 < self.filtered.len() {
                    self.selected.set(sel + 1);
                }
            }
            _ => {}
        }
        Route::Keep
    }

    fn send(self, tx: Sender<Self::Result>) {
        let result = self
            .filtered
            .get(self.selected.get())
            .copied();
        let _ = tx.send(result);
    }

    fn render(&self, f: &mut Frame, area: Rect, block: Block, is_focused: bool) {
        let inner = block.inner(area);
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).split(inner);

        let query_chars = self.query.chars();
        let query_byte_pos = self
            .query
            .char_indices()
            .nth(query_chars.count())
            .map(|(i, _)| i)
            .unwrap_or(self.query.len());
        let before = &self.query[..query_byte_pos];
        let after = &self.query[query_byte_pos..];

        let search_line = Line::from_iter([
            Span::raw("> "),
            Span::raw(before.to_string()),
            if is_focused {
                Span::raw(" ").reversed()
            } else {
                Span::raw(" ")
            },
            Span::raw(after.to_string()),
        ]);
        let prefix_width = UnicodeWidthStr::width(before) as u16 + 2;
        let search_para = Paragraph::new(search_line).scroll((
            0,
            prefix_width.saturating_sub(chunks[0].width.saturating_sub(4)),
        ));
        f.render_widget(search_para, chunks[0]);

        let highlight_style = Style::new().reversed();
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .enumerate()
            .map(|(fi, &orig_idx)| {
                let item = &self.items[orig_idx];
                let display = item.display();
                let pos_set = item.fuzzy_match(&self.query).unwrap_or_default();
                let mut spans = Vec::new();
                let mut last = 0;
                for &pos in &pos_set {
                    if pos > last {
                        spans.push(Span::raw(&display[last..pos]));
                    }
                    let end = pos + display[pos..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                    spans.push(Span::raw(&display[pos..end]).style(highlight_style));
                    last = end;
                }
                if last < display.len() {
                    spans.push(Span::raw(&display[last..]));
                }
                let line = Line::from(spans);
                let mut list_item = ListItem::new(line);
                if fi == self.selected.get() && is_focused {
                    list_item = list_item.style(highlight_style);
                }
                list_item
            })
            .collect();

        let mut list_state = ListState::default().with_selected(Some(self.selected.get()));
        let list = List::new(items).highlight_style(highlight_style);
        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }

    fn size(&self) -> (u16, u16) {
        let max_width = self
            .items
            .iter()
            .map(|i| i.display().len())
            .max()
            .unwrap_or(0)
            .max(10) as u16;
        let height = (self.filtered.len().max(1) + 1) as u16;
        (max_width, height.min(20))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_match_exact() {
        let result = "hello".fuzzy_match("hello");
        assert_eq!(result, Some(vec![0, 1, 2, 3, 4]));
    }

    #[test]
    fn fuzzy_match_substring() {
        let result = "MyProfile".fuzzy_match("pro");
        assert_eq!(result, Some(vec![2, 3, 4]));
    }

    #[test]
    fn fuzzy_match_case_insensitive() {
        let result = "MYPROFILE".fuzzy_match("pro");
        assert_eq!(result, Some(vec![2, 3, 4]));
    }

    #[test]
    fn fuzzy_match_no_match() {
        let result = "hello".fuzzy_match("xyz");
        assert_eq!(result, None);
    }

    #[test]
    fn fuzzy_match_sequential_chars() {
        let result = "abcdef".fuzzy_match("ace");
        assert_eq!(result, Some(vec![0, 2, 4]));
    }

    #[test]
    fn fuzzy_match_non_sequential_fails() {
        let result = "MyProfile".fuzzy_match("fpm");
        assert_eq!(result, None);
    }

    #[test]
    fn fuzzy_match_empty_query() {
        let result = "anything".fuzzy_match("");
        assert_eq!(result, Some(Vec::new()));
    }

    #[test]
    fn fuzzy_match_cjk() {
        let result = "你好世界".fuzzy_match("好世");
        assert_eq!(result, Some(vec![3, 6]));
    }
}
