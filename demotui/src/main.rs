mod config;
mod tui;

fn main() {
    config::Wrapper::init(None).unwrap();

    let app = tui::App::new();
    app.serve().unwrap();
}
