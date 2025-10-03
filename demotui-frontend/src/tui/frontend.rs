use anyhow::Result;

use tokio::sync::mpsc::{Receiver, Sender};

use demotui_shared::{frontend::event::FrontEndEvent, layer::Layer};

use crate::tui::core::{
    confirm::Confirm, help::Help, input::Input, popup::Popup, prof_tmpl::ProfTmpl,
    profile::Profile, service::Service, template::Template, which::Which,
};

pub struct FrontEnd {
    pub profile: Profile,
    pub template: Template,
    pub service: Service,
    pub input: Input,
    pub confirm: Confirm,
    pub popup: Popup,
    pub prof_tmpl: ProfTmpl, // profile and template
    pub which: Which,
    pub help: Help,
}

impl FrontEnd {
    pub fn new() -> Self {
        Self {
            profile: Default::default(),
            template: Default::default(),
            service: Default::default(),
            input: Default::default(),
            confirm: Default::default(),
            popup: Default::default(),
            prof_tmpl: Default::default(),
            which: Default::default(),
            help: Default::default(),
        }
    }

    pub fn layer(&self) -> Layer {
        if self.which.visible {
            Layer::Which
        } else if self.help.visible {
            Layer::Help
        } else if self.popup.visible {
            Layer::Popup
        } else if self.confirm.visible {
            Layer::Confirm
        } else if self.input.visible {
            Layer::Input
        } else {
            Layer::Main
        }
    }
}
