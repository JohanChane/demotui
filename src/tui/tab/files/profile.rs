use super::*;

pub enum Key {
    Select,
    Delete,
    Preview,
    Update,

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
            KeyCode::Enter => Self::Select,
            KeyCode::Char('d') => Self::Delete,
            KeyCode::Char('p') => Self::Preview,
            KeyCode::Char('u') => Self::Update,

            KeyCode::Down => Self::MoveDown,
            KeyCode::Up => Self::MoveUp,

            _ => return Err(()),
        })
    }
}

#[derive(Default)]
pub struct Profile {
    profiles: Vec<String>,
    atime: Vec<Option<Duration>>,
    filter: Option<String>,
    pub is_focused: bool,
}

impl TabContent for Profile {
    type Key = Key;
    type State = ListState;

    const TITLE: &str = "profile";

    fn handle_key_event(
        &mut self,
        key: Self::Key,
        task_set: &mut FutureSet<Self>,
        state: &mut Self::State,
    ) {
        todo!()
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State) {
        let list = List::from_iter(
            self.profiles
                .iter()
                .zip(self.atime.iter())
                // filter content now
                .filter(|(value, _)| self.filter.as_deref().is_none_or(|pat| value.contains(pat)))
                .map(|(value, extra)| {
                    ListItem::new(Line::from(vec![
                        Span::raw(value),
                        Span::raw("("),
                        Span::raw("").style(Theme::get().profile_tab.update_interval),
                        Span::raw(")"),
                    ]))
                }),
        )
        .block(
            Block::bordered()
                .border_style(if self.is_focused {
                    Theme::get().list.block_selected
                } else {
                    Theme::get().list.block_unselected
                })
                .title(Self::TITLE),
        )
        .highlight_style(if self.is_focused {
            Theme::get().list.highlight
        } else {
            Theme::get().list.unhighlight
        });

        f.render_stateful_widget(list, area, state);
    }
}
