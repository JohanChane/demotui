use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};

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
    pub const TITLE: &str = "File";
}

crate::new_type_impl_tuiwidget!(FileTab);
