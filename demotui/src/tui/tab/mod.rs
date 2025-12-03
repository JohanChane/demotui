mod files;

mod dev {
    pub use crate::tui::widget::tab::{FutureSet, Tab, TabContent, wrapper};
    pub use crossterm::event::{KeyCode, KeyEvent};
    pub use ratatui::prelude::{Frame, Rect};
    pub use ratatui::widgets::{Block, List, ListState, StatefulWidget};

    pub use crate::tui::popmsg::prelude::*;
    pub use crate::tui::theme::Theme;
}

pub mod prelude {
    pub use super::files::FileTab;
}
