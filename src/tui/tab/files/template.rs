use super::*;
use crate::functions::file::template::*;

pub enum Key {
    Generate,
    Delete,
    Preview,

    Switch,
    MoveUp,
    MoveDown,
}

impl TryFrom<&KeyEvent> for Key {
    type Error = ();

    fn try_from(value: &KeyEvent) -> Result<Self, Self::Error> {
        if value.kind != crossterm::event::KeyEventKind::Press {
            return Err(());
        }
        Ok(match value.code {
            KeyCode::Enter => Self::Generate,
            KeyCode::Char('d') => Self::Delete,
            KeyCode::Char('p') => Self::Preview,

            KeyCode::Right | KeyCode::Left => Self::Switch,
            KeyCode::Down => Self::MoveDown,
            KeyCode::Up => Self::MoveUp,

            _ => return Err(()),
        })
    }
}

#[derive(Default)]
pub struct Template {
    items: Vec<String>,
    filter: Option<String>,
}

impl BasicTabContent for Template {
    type Key = Key;
    type State = ListState;

    const TITLE: &str = "Template";
}

impl DualTabContentMate for Template {
    type Mate = super::profile::Profile;

    fn init(&mut self, task_set: &mut FutureSet<(Self::Mate, Self)>, _: &mut Self::State) {
        async {
            let templates = tri!(get_all_templates());
            wrapper(|(_, content): &mut (Self::Mate, Self)| content.items = templates)
        }
        .spawn_at(task_set);
    }

    fn handle_key_event(
        &mut self,
        key: Self::Key,
        task_set: &mut FutureSet<(Self::Mate, Self)>,
        state: &mut Self::State,
    ) -> bool {
        match key {
            Key::Generate => {
                let name = get_name!(self, state);
                async {
                    tri!(apply_template(name));

                    profile_sync!(%)
                }
                .spawn_at(task_set);
            }
            Key::Delete => todo!(),
            Key::Preview => todo!(),

            Key::Switch => return true,
            Key::MoveDown => state.select_next(),
            Key::MoveUp => state.select_previous(),
        }
        false
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State, is_focused: bool) {
        let block = Block::bordered()
            .border_style(if is_focused {
                Theme::get().tab.tab_focused
            } else {
                Theme::get().tab.dualtab_unfocused
            })
            .title(Self::TITLE);

        let block = if let Some(filter) = self.filter.as_ref() {
            block.title_bottom(Line::raw(format!(" {filter} ")).right_aligned().reversed())
        } else {
            block.title_bottom(Line::raw(format!(" /: Search ")).right_aligned().reversed())
        };

        let iter = self
            .items
            .iter()
            // filter content now
            .filter_map(|value| {
                self.filter
                    .as_deref()
                    .is_none_or(|pat| value.contains(pat))
                    .then_some(value.as_str())
            });
        let widget = List::from_iter(iter)
            .block(block)
            .highlight_style(if is_focused {
                Theme::get().tab.item_highlighted
            } else {
                Theme::get().tab.item_unhighlighted
            });
        f.render_stateful_widget(widget, area, state);
    }
}
