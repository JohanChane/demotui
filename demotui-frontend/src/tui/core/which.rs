use demotui_config::Key;
use demotui_shared::{layer::Layer, render, render_and};

#[derive(Default)]
pub(crate) struct Which {
    pub visible: bool,
}

impl Which {
    pub fn r#type(&mut self, key: Key) -> bool {
        render_and!(true)
    }

    fn reset(&mut self) {}

    pub fn show_with(&mut self, key: Key, layer: Layer) {
        render!();
    }
}
