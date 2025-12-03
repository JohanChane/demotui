use super::*;

pub enum Key {}

impl TryFrom<&KeyEvent> for Key {
    type Error = ();

    fn try_from(value: &KeyEvent) -> Result<Self, Self::Error> {
        todo!()
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
                        Span::raw(""), //.style(Theme::get().profile_tab.update_interval),
                        Span::raw(")"),
                    ]))
                }),
        );
        // .block(
        //     Raw::Block::default()
        //         .borders(Raw::Borders::ALL)
        //         .border_style(if is_fouced {
        //             Theme::get().list.block_selected
        //         } else {
        //             Theme::get().list.block_unselected
        //         })
        //         .title(self.title.as_str()),
        // )
        // .highlight_style(if is_fouced {
        //     Theme::get().list.highlight
        // } else {
        //     Theme::get().list.unhighlight
        // })

        f.render_stateful_widget(list, area, state);
    }
}
