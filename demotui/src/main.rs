mod config;
mod tui;

fn main() {
    config::init(None).unwrap();

    let app = tui::App::new();
    app.serve().unwrap();
}
