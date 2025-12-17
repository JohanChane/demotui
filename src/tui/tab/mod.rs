mod dev {
    pub use crate::tui::widget::dualtab::*;
    pub use crate::tui::widget::tab::*;
    pub use crossterm::event::{KeyCode, KeyEvent};
    pub use ratatui::prelude::{Frame, Rect};
    pub use ratatui::style::Stylize as _;
    pub use ratatui::widgets::{Block, List, ListState, StatefulWidget};

    pub use crate::tui::popmsg::prelude::*;
    pub use crate::tui::theme::Theme;
}

macro_rules! tri {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                crate::tui::widget::popmsg::Confirm::err(e);
                return do_nothing();
            }
        }
    };
    ($e:expr, or_cancel) => {
        match $e {
            Ok(v) => v,
            Err(_) => {
                return do_nothing();
            }
        }
    };
}

mod files;
mod status;

pub mod prelude {
    pub use super::files::FileTab;
    pub use super::status::StatusTab;
}
