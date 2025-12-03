use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};
use std::time::Duration;

use crate::tui::TuiWidget;

use super::dev::*;

mod profile;
mod template;

pub struct FileTab {
    profile: Tab<profile::Profile>,
    template: Tab<template::Template>,
    pub is_focus_profile: bool,
}

impl FileTab {
    const TITLES: [&str; 2] = [profile::Profile::TITLE, template::Template::TITLE];
}

impl Default for FileTab {
    fn default() -> Self {
        Self {
            profile: Default::default(),
            template: Default::default(),
            is_focus_profile: true,
        }
    }
}

impl TuiWidget for FileTab {
    fn handle_key_event(&mut self, kv: &KeyEvent) {
        if self.is_focus_profile {
            self.profile.handle_key_event(kv);
        } else {
            self.template.handle_key_event(kv);
        }
    }

    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::layout::{Constraint::Ratio, Layout};

        let cons = if self.is_focus_profile {
            [Ratio(7, 10), Ratio(3, 10)]
        } else {
            [Ratio(3, 10), Ratio(7, 10)]
        };
        let hori = Layout::horizontal(cons).split(area);

        self.profile.content_mut().is_focused = self.is_focus_profile;
        self.template.content_mut().is_focused = !self.is_focus_profile;

        self.profile.render(f, hori[0]);
        self.template.render(f, hori[1]);
    }
}
