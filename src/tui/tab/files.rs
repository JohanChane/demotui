use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};

use crate::new_type_impl_tuiwidget;

use super::dev::*;

#[macro_use]
mod profile;
mod template;

/// This can only be [DualTab], because [Template] needs to update [Profile]
///
/// [Template]: template::Template
/// [Profile]: profile::Profile
#[derive(Default)]
pub struct FileTab(DualTab<profile::Profile, template::Template>);

impl FileTab {
    pub const TITLES: [&str; 2] = [profile::Profile::TITLE, template::Template::TITLE];

    pub fn focused_on_profile(&mut self) {
        self.0.focused_on_c1();
    }
    pub fn focused_on_template(&mut self) {
        self.0.focused_on_c2();
    }
}

new_type_impl_tuiwidget!(FileTab);
