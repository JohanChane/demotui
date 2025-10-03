use crate::tui::core::{profile::Profile, template::Template};

#[derive(Default)]
enum Fouce {
    #[default]
    Profile,
    Template,
}

#[derive(Default)]
pub(crate) struct ProfTmpl {
    pub visible: bool,
    pub profile: Profile,
    pub template: Template,
    pub fouce: Fouce,
}

impl ProfTmpl {
    // fn r#type(&self, key) {
    //     match self.fouce {
    //         Profile => {},
    //         Template => {}
    //     }
    // }

    // fn get_keymap() {
    //     match self.fouce {
    //         Profile => {}
    //         Template => {}
    //     }
    // }
}
