pub mod backend;
pub mod consts;
pub mod data;
pub mod frontend;
pub mod macros;
pub mod ro_cell;
pub mod utils;

mod_flat!(alias);

use crate::utils::log_utils;

pub fn init() {
    backend::event::BackEndEvent::init();
    log_utils::setup(3);
    frontend::event::FrontEndEvent::init();
}
