use std::ops::Deref;

use anyhow::Result;

use super::Chord;
use crate::Key;

pub struct KeymapRules {
    pub keymap: Vec<Chord>,
}

impl Deref for KeymapRules {
    type Target = Vec<Chord>;

    fn deref(&self) -> &Self::Target {
        &self.keymap
    }
}
