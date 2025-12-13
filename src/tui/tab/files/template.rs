use super::*;
use crate::functions::file::template::*;

pub enum Key {
    Generate,
    Delete,
    Preview,

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

            KeyCode::Down => Self::MoveDown,
            KeyCode::Up => Self::MoveUp,

            _ => return Err(()),
        })
    }
}

macro_rules! get_name {
    ($self:expr,$state:expr) => {
        if let Some(idx) = $state.selected() {
            $self.templates[idx].clone()
        } else {
            return;
        }
    };
}

#[derive(Default)]
pub struct Template {
    templates: Vec<String>,
    filter: Option<String>,
}

impl BasicTabContent for Template {
    type Key = Key;
    type State = ListState;

    const TITLE: &str = "template";
}

impl DualTabContentMate for Template {
    type Mate = super::profile::Profile;

    fn init(&mut self, task_set: &mut FutureSet<(Self::Mate, Self)>, _: &mut Self::State) {
        async {
            let templates = tri!(get_all_templates());
            wrapper(|(_, content): &mut (Self::Mate, Self)| content.templates = templates)
        }
        .spawn(task_set);
    }

    fn handle_key_event(
        &mut self,
        key: Self::Key,
        task_set: &mut FutureSet<(Self::Mate, Self)>,
        state: &mut Self::State,
    ) {
        match key {
            Key::MoveDown => state.select_next(),
            Key::MoveUp => state.select_previous(),
            Key::Generate => {
                let name = get_name!(self, state);
                async {
                    tri!(template::apply_template(name));

                    profile_sync!(%)
                }
                .spawn(task_set);
            }
            Key::Delete => todo!(),
            Key::Preview => todo!(),
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State, is_focused: bool) {
        let list = List::from_iter(
            self.templates
                .iter()
                // filter content now
                .filter_map(|value| {
                    self.filter
                        .as_deref()
                        .is_none_or(|pat| value.contains(pat))
                        .then_some(value.as_str())
                }),
        )
        .block(
            Block::bordered()
                .border_style(if is_focused {
                    Theme::get().list.block_selected
                } else {
                    Theme::get().list.block_unselected
                })
                .title(Self::TITLE),
        )
        .highlight_style(if is_focused {
            Theme::get().list.highlight
        } else {
            Theme::get().list.unhighlight
        });

        f.render_stateful_widget(list, area, state);
    }
}
