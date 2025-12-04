mod config;
mod tui;

fn main() {
    config::init(None).unwrap();
    tui::init().unwrap();

    let app = tui::App::new();
    app.serve().unwrap();

    tui::restore().unwrap();
}
