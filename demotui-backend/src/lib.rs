use demotui_shared::mod_flat;

pub mod actor;
pub mod backend;
pub mod dispatcher;

mod_flat!(actors);

pub mod context;
pub mod executor;
