use anyhow::{Context, Result};
use crossterm::event::KeyCode;

use super::Chord;
use super::KeymapRules;

pub struct Keymap {
    pub profile: KeymapRules,
    // pub template: KeymapRules,
    // pub service: KeymapRules,
}

impl Keymap {
    pub(super) fn new() -> Self {
        let profile = Keymap::make_profile_km_rules();

        Self { profile }
    }

    fn make_profile_km_rules() -> KeymapRules {
        let profile_keymap = KeymapRules {
            keymap: vec![
                Chord::new(KeyCode::Char('j'), Some("next".into())),
                Chord::new(KeyCode::Char('k'), Some("previous".into())),
                Chord::new(KeyCode::Char('u'), Some("update profile".into())),
                Chord::new(KeyCode::Enter, Some("select profile".into())),
            ],
        };

        profile_keymap
    }
}
