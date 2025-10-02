mod app;
mod dispatcher;
mod executor;
mod signals;
mod term;

use demotui_backend::backend::BackEnd;
use demotui_frontend::tui::frontend::{self, FrontEnd};

use anyhow::Result;

use crate::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    App::serve().await
}
