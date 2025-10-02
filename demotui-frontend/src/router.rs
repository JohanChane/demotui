use anyhow::Result;
use demotui_shared::layer::Layer;

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

    //     match layer {
    //         Layer::App => unreachable!(),

    //         Layer::Which => core.which.r#type(),
    //         // ...
    //         Layer::Main => {
    //             // check tab index
    //             // if prof_tmpl:
    //             //      prof_tmpl.r#type()
    //         }
    //     }
    // }

    // fn matches(&mut self, layer: Layer, key: Key) -> bool {
    //     for chord @ Chord { on, .. } in KEYMAP.get(layer) {
    //         if on.is_empty() || on[0] != key {
    //             continue;
    //         }

    //         if on.len() > 1 {
    //             self.app.core.which.show_with(key, layer);
    //         } else {
    //             emit!(Seq(ChordCow::from(chord).into_seq()));
    //         }
    //         return true;
    //     }
    //     false
    // }
}
