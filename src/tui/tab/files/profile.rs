use crate::functions::command::{edit, test_config};
use crate::functions::file::profile::{db, select, update_profile};
use crate::tui::widget::popmsg::Confirm;

use super::*;

pub enum Key {
    Add,
    Select,
    Delete,
    Edit,
    Preview,
    Update,
    Search,
    Test,

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
            KeyCode::Char('i') => Self::Add,
            KeyCode::Char('e') => Self::Edit,
            KeyCode::Enter => Self::Select,
            KeyCode::Char('d') => Self::Delete,
            KeyCode::Char('p') => Self::Preview,
            KeyCode::Char('u') => Self::Update,
            KeyCode::Char('/') => Self::Search,
            KeyCode::Char('t') => Self::Test,

            KeyCode::Down => Self::MoveDown,
            KeyCode::Up => Self::MoveUp,

            _ => return Err(()),
        })
    }
}

macro_rules! get_name {
    ($self:expr,$state:expr) => {
        if let Some(idx) = $state.selected() {
            $self.profiles[idx].clone()
        } else {
            return;
        }
    };
}

macro_rules! sync {
    ($content:expr) => {{
        let mut composed: Vec<(String, Option<std::time::Duration>)> =
            crate::functions::file::profile::db::get_all()
                .into_iter()
                .map(|pf| {
                    (
                        pf.name.clone(),
                        pf.load_local_profile().ok().and_then(|lp| lp.atime()),
                    )
                })
                .collect();
        composed.sort_unstable();
        let (name, atime) = composed.into_iter().unzip();
        $content.atime = atime;
        $content.profiles = name;
    }};
    () => {
        wrapper(|content: &mut Self| sync!(content))
    };
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
        match key {
            Key::Add => {
                async {
                    let name = tri!(
                        Input::new()
                            .with_title("Name".to_owned())
                            .build_and_send()
                            .await,
                        or_cancel
                    );
                    let url = tri!(
                        Input::new()
                            .with_title("Url".to_owned())
                            .build_and_send()
                            .await,
                        or_cancel
                    );
                    let pf = tri!(db::create(name, url));
                    tri!(update_profile(pf, false, false).await);

                    sync!()
                }
                .spawn(task_set);
            }
            Key::Edit => {
                let name = get_name!(self, state);
                async {
                    let pf = tri!(db::get(name).unwrap().load_local_profile());
                    tri!(edit(pf.path.to_str().unwrap()));

                    do_nothing()
                }
                .spawn(task_set);
            }
            Key::Select => {
                let name = get_name!(self, state);
                async {
                    tri!(select(db::get(name).unwrap()));
                    do_nothing()
                }
                .spawn(task_set);
            }
            Key::Delete => {
                let name = get_name!(self, state);
                async {
                    let pf = db::get(name).unwrap();
                    tri!(db::remove(pf));

                    sync!()
                }
                .spawn(task_set);
            }
            Key::Preview => {
                let name = get_name!(self, state);
                async {
                    let mut lines = Vec::with_capacity(512);
                    let pf = tri!(db::get(name).unwrap().load_local_profile());
                    lines.push(
                        pf.dtype
                            .get_domain()
                            .unwrap_or("Imported local file".to_owned()),
                    );
                    lines.push(Default::default());

                    let content = tri!(std::fs::read_to_string(pf.path));
                    if content.is_empty() {
                        lines.push("yaml file is empty. Please update it.".to_owned());
                    } else {
                        lines.extend(content.lines().map(|s| s.to_owned()));
                    }

                    do_nothing()
                }
                .spawn(task_set);
            }
            Key::Update => {
                let name = get_name!(self, state);
                async {
                    let with_proxy = todo!();
                    let remove_proxy_provider = todo!();
                    let result = tri!(
                        update_profile(db::get(name).unwrap(), with_proxy, remove_proxy_provider,)
                            .await
                    );

                    sync!()
                }
                .spawn(task_set);
            }
            Key::Search => {
                async {
                    let filter = tri!(
                        Input::new()
                            .with_title("Filter".to_owned())
                            .build_and_send()
                            .await,
                        or_cancel
                    );

                    wrapper(|content: &mut Self| {
                        content.filter = Some(filter);
                    })
                }
                .spawn(task_set);
            }
            Key::Test => {
                let name = get_name!(self, state);
                async {
                    let enable_geodata_mode = todo!();
                    let pf = tri!(db::get(name).unwrap().load_local_profile());
                    let result = test_config(Some(&pf.path), enable_geodata_mode);
                    Confirm::title("Test Result".to_owned())
                        .with_prompt(result)
                        .build_and_send();

                    do_nothing()
                }
                .spawn(task_set);
            }

            Key::MoveUp => state.select_previous(),
            Key::MoveDown => state.select_next(),
        }
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
