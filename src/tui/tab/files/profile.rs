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

/// The Only reason why I use two functions to `sync` is that
/// I except modifying Self (what we do in `wrapper`) is
/// fast and infallable
///
/// Tasks should be done in async{} and left only values that
/// apply to Self
macro_rules! profile_sync {
    () => {{
        let (name, atime) = super::profile::get_profiles_with_readable_atime();
        wrapper(|(content, _): &mut (Self, Self::Mate)| {
            super::profile::sync_helper(content, name, atime)
        })
    }};
    (%) => {{
        let (name, atime) = super::profile::get_profiles_with_readable_atime();
        wrapper(|(content, _): &mut (Self::Mate, Self)| {
            super::profile::sync_helper(content, name, atime)
        })
    }};
}

#[derive(Default)]
pub struct Profile {
    profiles: Vec<String>,
    // atime: Vec<Option<Duration>>,
    atime: Vec<String>,
    filter: Option<String>,
}

impl BasicTabContent for Profile {
    type Key = Key;
    type State = ListState;

    const TITLE: &str = "profile";
}

impl DualTabContent for Profile {
    type Mate = super::template::Template;

    fn init(&mut self, task_set: &mut FutureSet<(Self, Self::Mate)>, _: &mut Self::State) {
        async { profile_sync!() }.spawn_at(task_set);
    }

    fn handle_key_event(
        &mut self,
        key: Self::Key,
        task_set: &mut FutureSet<(Self, Self::Mate)>,
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

                    profile_sync!()
                }
                .spawn_at(task_set);
            }
            Key::Edit => {
                let name = get_name!(self, state);
                async {
                    let pf = tri!(db::get(name).unwrap().load_local_profile());
                    tri!(edit(pf.path.to_str().unwrap()));

                    do_nothing()
                }
                .spawn_at(task_set);
            }
            Key::Select => {
                let name = get_name!(self, state);
                async {
                    tri!(select(db::get(name).unwrap()));
                    do_nothing()
                }
                .spawn_at(task_set);
            }
            Key::Delete => {
                let name = get_name!(self, state);
                async {
                    let pf = db::get(name).unwrap();
                    tri!(db::remove(pf));

                    profile_sync!()
                }
                .spawn_at(task_set);
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

                    Confirm::title("Preview".to_owned())
                        .with_prompt(lines.join("\n"))
                        .build_and_send();

                    do_nothing()
                }
                .spawn_at(task_set);
            }
            Key::Update => {
                let name = get_name!(self, state);
                async {
                    let with_proxy = todo!("crate::tui::popmsg::SelectSingle");
                    let remove_proxy_provider = todo!("crate::tui::popmsg::SelectSingle");
                    let result = tri!(
                        update_profile(db::get(name).unwrap(), with_proxy, remove_proxy_provider,)
                            .await
                    );

                    profile_sync!()
                }
                .spawn_at(task_set);
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

                    wrapper(|(content, _): &mut (Self, Self::Mate)| {
                        content.filter = Some(filter);
                    })
                }
                .spawn_at(task_set);
            }
            Key::Test => {
                let name = get_name!(self, state);
                async {
                    let enable_geodata_mode = todo!("crate::tui::popmsg::SelectSingle");
                    let pf = tri!(db::get(name).unwrap().load_local_profile());
                    let result = test_config(Some(&pf.path), enable_geodata_mode);
                    Confirm::title("Test Result".to_owned())
                        .with_prompt(result)
                        .build_and_send();

                    do_nothing()
                }
                .spawn_at(task_set);
            }

            Key::MoveUp => state.select_previous(),
            Key::MoveDown => state.select_next(),
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State, is_focused: bool) {
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
                        Span::raw(extra).style(Theme::get().profile_tab.update_interval),
                        Span::raw(")"),
                    ]))
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

pub(super) fn get_profiles_with_readable_atime() -> (Vec<String>, Vec<String>) {
    let mut composed: Vec<(String, String)> = crate::functions::file::profile::db::get_all()
        .into_iter()
        .map(|pf| {
            (
                pf.name.clone(),
                pf.load_local_profile()
                    .ok()
                    .and_then(|lp| lp.atime())
                    .map(display_duration)
                    .unwrap_or_else(|| "Unknown".to_owned()),
            )
        })
        .collect();
    composed.sort_unstable();
    let (name, atime) = composed.into_iter().unzip();
    (name, atime)
}

pub(super) fn sync_helper(content: &mut Profile, name: Vec<String>, atime: Vec<String>) {
    content.atime = atime;
    content.profiles = name;
}

fn display_duration(t: std::time::Duration) -> String {
    use std::time::Duration;
    if t.is_zero() {
        "Just Now".to_string()
    } else if t < Duration::from_secs(60 * 59) {
        let min = t.as_secs() / 60;
        format!("In {} mins", min + 1)
    } else if t < Duration::from_secs(3600 * 24) {
        let hou = t.as_secs() / 3600;
        format!("In {hou} hours")
    } else {
        let day = t.as_secs() / (3600 * 24);
        format!("In about {day} days")
    }
}
