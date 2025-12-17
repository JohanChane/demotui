use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};

use super::dev::*;

#[macro_use]
mod profile;
mod template;

newtype_tab!(
    /// This can only be [DualTab], because [Template] needs to update [Profile]
    ///
    /// [Template]: template::Template
    /// [Profile]: profile::Profile
    FileTab(DualTab<profile::Profile, template::Template>),
    "File"
);
