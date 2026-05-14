use super::dev::*;
use ratatui::{
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Clear, ListItem, Paragraph},
};
use strum::VariantArray;

newtype_tab!(SettingsTab(Tab<SettingsContent>));

mod_agent!(
    SettingsKey,
    [
        ([KeyCode::Enter], SettingsKey::Execute, "Apply"),
        ([KeyCode::Esc], SettingsKey::Esc, "Back"),
        ([KeyCode::Up], SettingsKey::MoveUp, "Move up"),
        ([KeyCode::Down], SettingsKey::MoveDown, "Move down"),
        ([KeyCode::Char('k')], SettingsKey::MoveUp, "Move up"),
        ([KeyCode::Char('j')], SettingsKey::MoveDown, "Move down"),
    ]
);

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) enum SettingsKey {
    Execute,
    MoveUp,
    MoveDown,
    Esc,
}

impl TryFrom<&crate::tui::Key> for SettingsKey {
    type Error = ();

    fn try_from(ev: &crate::tui::Key) -> Result<Self, Self::Error> {
        let agent = agent();
        if !agent.is_empty() {
            return agent.get(ev).map(|k| *k).ok_or(());
        }
        Ok(match ev.code {
            KeyCode::Enter => Self::Execute,
            KeyCode::Esc => Self::Esc,
            KeyCode::Up | KeyCode::Char('k') => Self::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => Self::MoveDown,
            _ => return Err(()),
        })
    }
}

use crate::functions::restful::config_struct::Mode;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsOp {
    SwitchMode,
}

impl SettingsOp {
    fn all() -> Vec<Self> {
        vec![Self::SwitchMode]
    }
}

#[derive(Default)]
struct SettingsContent {
    ops: Vec<SettingsOp>,
    current_mode: String,
    mode_selector_state: ListState,
    mode_selector_visible: bool,
    modes: Vec<Mode>,
}

impl BasicTabContent for SettingsContent {
    type Key = SettingsKey;

    type State = ListState;

    const TITLE: &str = "Settings";

    fn all_shortcuts() -> &'static [(KeyCombo, Self::Key, &'static str)] {
        agent::all_shortcuts()
    }

    fn on_enter(&mut self, _task_set: &mut FutureSet<Self>, _state: &mut Self::State) {
        if crate::config::is_core_mismatch() {
            self.current_mode = "core mismatch".to_owned();
        }
    }
}

impl TabContent for SettingsContent {
    fn init(&mut self, task_set: &mut FutureSet<Self>, state: &mut Self::State) {
        self.ops = SettingsOp::all();
        self.modes = Mode::VARIANTS.to_vec();
        self.mode_selector_state.select(Some(0));
        if !self.ops.is_empty() {
            state.select(Some(0));
        }
        if crate::config::is_core_mismatch() {
            self.current_mode = "core mismatch".to_owned();
            return;
        }

        async move {
            let result =
                tokio::task::spawn_blocking(|| {
                    crate::functions::restful::config::fetch()
                })
                .await
                .unwrap();
            match result {
                Ok(config) => {
                    let mode = config.mode.to_string();
                    wrapper(move |c: &mut SettingsContent| {
                        c.current_mode = mode;
                    })
                }
                Err(e) => {
                    crate::tui::widget::popmsg::Confirm::err(e);
                    do_nothing()
                }
            }
        }
        .spawn_at(task_set);
    }

    fn handle_key_event(
        &mut self,
        key: SettingsKey,
        task_set: &mut FutureSet<Self>,
        state: &mut Self::State,
    ) {
        if self.mode_selector_visible {
            match key {
                SettingsKey::MoveUp => {
                    let i = self.mode_selector_state.selected().unwrap_or(0);
                    self.mode_selector_state
                        .select(Some(i.saturating_sub(1)));
                }
                SettingsKey::MoveDown => {
                    let i = self.mode_selector_state.selected().unwrap_or(0);
                    if i + 1 < self.modes.len() {
                        self.mode_selector_state.select(Some(i + 1));
                    }
                }
                SettingsKey::Esc => {
                    self.mode_selector_visible = false;
                }
                SettingsKey::Execute => {
                    let idx =
                        self.mode_selector_state.selected().unwrap_or(0);
                    if let Some(mode) = self.modes.get(idx) {
                        let mode = *mode;
                        self.mode_selector_visible = false;
                        async move {
                            if crate::config::is_core_mismatch() {
                                return do_nothing();
                            }
                            let payload =
                                serde_json::json!({"mode": mode.to_string()})
                                    .to_string();
                            let result = tokio::task::spawn_blocking(move || {
                                crate::functions::restful::config::patch(
                                    payload,
                                )
                            })
                            .await
                            .unwrap();
                            match result {
                                Ok(_) => {
                                    let new_val = mode.to_string();
                                    wrapper(move |c: &mut SettingsContent| {
                                        c.current_mode = new_val;
                                    })
                                }
                                Err(e) => {
                                    crate::tui::widget::popmsg::Confirm::err(
                                        e,
                                    );
                                    do_nothing()
                                }
                            }
                        }
                        .spawn_at(task_set);
                    }
                }
            }
            return;
        }

        match key {
            SettingsKey::MoveUp => {
                let i = state.selected().unwrap_or(0);
                state.select(Some(i.saturating_sub(1)));
            }
            SettingsKey::MoveDown => {
                let i = state.selected().unwrap_or(0);
                if i + 1 < self.ops.len() {
                    state.select(Some(i + 1));
                }
            }
            SettingsKey::Execute => {
                let Some(idx) = state.selected() else { return };
                let Some(op) = self.ops.get(idx) else { return };
                match op {
                    SettingsOp::SwitchMode => {
                        self.mode_selector_visible = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State) {
        let block = Block::bordered()
            .border_style(Theme::get().section("settings").border)
            .title(Self::TITLE);

        if crate::config::is_core_mismatch() {
            let widget = Paragraph::new("API data mismatch with configured core").block(block);
            f.render_widget(widget, area);
            return;
        }

        let value_style = Theme::get().section("settings").muted;

        let items: Vec<ListItem> = self
            .ops
            .iter()
            .map(|op| {
                let (name, current) = match op {
                    SettingsOp::SwitchMode => {
                        ("Mode", self.current_mode.as_str())
                    }
                };
                ListItem::new(Line::from(vec![
                    Span::raw(format!("  {:<14}", name)),
                    Span::raw(current).style(value_style),
                ]))
            })
            .collect();

        let highlight_style = Theme::get().section("settings").highlight;
        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style);

        f.render_stateful_widget(list, area, state);

        if self.mode_selector_visible {
            let select_area = centered_rect(60, 30, area);
            let mode_items: Vec<ListItem> = self
                .modes
                .iter()
                .map(|m| ListItem::new(format!("  {}", m)))
                .collect();
            let mode_list = List::new(mode_items)
                .block(
                    Block::bordered()
                        .border_style(Theme::get().section("settings").border)
                        .title("Mode"),
                )
                .highlight_style(highlight_style);
            f.render_widget(Clear, select_area);
            f.render_stateful_widget(
                mode_list,
                select_area,
                &mut self.mode_selector_state.clone(),
            );
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
