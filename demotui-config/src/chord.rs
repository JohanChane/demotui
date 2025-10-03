use crossterm::event::KeyCode;

use crate::key::Key;

pub struct Chord {
    pub on: Vec<Key>,
    // pub run: Vec<Cmd>,           // TODO
    pub desc: Option<String>,
    // pub r#for: Option<String>,
}

impl Chord {
    pub(super) fn new(keycode: KeyCode, desc: Option<String>) -> Self {
        Self {
            on: { vec![generate_key(keycode)] },
            desc,
        }
    }
}

fn generate_key(code: KeyCode) -> Key {
    Key {
        code,
        shift: false,
        ctrl: false,
        alt: false,
        super_: false,
    }
}
