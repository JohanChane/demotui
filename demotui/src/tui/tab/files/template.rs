use super::*;

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

#[derive(Default)]
pub struct Template {
    templates: Vec<String>,
    filter: Option<String>,
    pub is_focused: bool,
}

impl TabContent for Template {
    type Key = Key;
    type State = ListState;

    const TITLE: &str = "template";

    fn handle_key_event(
        &mut self,
        key: Self::Key,
        task_set: &mut FutureSet<Self>,
        state: &mut Self::State,
    ) {
        match key {
            Key::MoveDown => state.select_next(),
            Key::MoveUp => state.select_previous(),

            Key::Generate => {
                let name = if let Some(idx) = state.selected() {
                    self.templates[idx].clone()
                } else {
                    return;
                };
                let task = async {
                    println!("Start {}", name);
                    let rx = Input::new()
                        .with_title("Test input".to_owned())
                        .with_prompt("This is used to test input widget".to_owned())
                        .build_and_send();
                    let Ok(input_content) = rx.await else {
                        return wrapper(|_| anyhow::bail!("Task Canceled"));
                    };
                    wrapper(move |content: &mut Self| {
                        println!("Done {}, Input: {}", name, input_content);
                        Ok(())
                    })
                };
                task_set.spawn(task);
            }
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State) {
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
