use std::cell::LazyCell;

use anyhow::Result;
use demotui_config::{Chord, Key};
use demotui_shared::{frontend, layer::Layer};

use crate::tui::frontend::FrontEnd;
pub(super) struct Router<'a> {
    frontend: &'a mut FrontEnd,
}

impl<'a> Router<'a> {
    pub(super) fn new(frontend: &'a mut FrontEnd) -> Self {
        Self { frontend }
    }

    // pub(super) fn route(&mut self, key: Key) -> Result<bool> {
    //     let layer = self.frontend.layer();

    //     // match layer {
    //     //     Layer::App => unreachable!(),

    //     //     Layer::Popup => self.matches(layer, key)?;
    //     //     // ...

    //     //     // ...
    //     //     // Layer::Main => {
    //     //     //         check tab index
    //     //     //         if prof_tmpl:
    //     //     //              prof_tmpl.r#type()
    //     //     //     }

    //     //     Layer::Which => self.frontend.which.r#type(key)?,
    //     // }
    // }

    // fn matches(&mut self, layer: Layer, key: Key) -> bool {
    //     for chord @ Chord { on, .. } in KEYMAP.get(layer) {
    //         if on.is_empty() || on[0] != key {
    //             continue;
    //         }

    //         // get the chord
    //         if on.len() > 1 {
    //             self.frontend.which.show_with(key, layer); // 根据已按的 key 显示已经匹配的 chord
    //         } else {
    //             // emit!(Seq(ChordCow::from(chord).into_seq()));    // emit chord 对应的 run (cmds or ops)
    //         }
    //         return true;
    //     }
    //     false
    // }
}
