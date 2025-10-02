use anyhow::Result;
pub(super) struct Router {}

impl Router {
    pub(super) fn new() -> Self {
        Self {}
    }

    // pub(super) fn route(&mut self, key: Key) -> Result<bool> {
    //     let core = &mut self.app.core;
    //     let layer = core.layer();

    //     if core.help.visible && core.help.r#type(&key)? {
    //         return Ok(true);
    //     }
    //     if core.input.visible && core.input.r#type(&key)? {
    //         return Ok(true);
    //     }

    //     use Layer as L;
    //     Ok(match layer {
    //         L::App => unreachable!(),
    //         L::Mgr | L::Tasks | L::Spot | L::Pick | L::Input | L::Confirm | L::Help => {
    //             self.matches(layer, key)
    //         }
    //         L::Cmp => self.matches(L::Cmp, key) || self.matches(L::Input, key),
    //         L::Which => core.which.r#type(key),
    //     })
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
