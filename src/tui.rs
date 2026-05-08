use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use utils::*;

mod agent;
mod app;
mod key;
mod popmsg;
mod signals;
mod tab;
mod theme;
mod utils;
mod widget;

pub use app::App;
pub use key::Key;
pub use theme::Theme;

static CSI_U_ENABLED: AtomicBool = AtomicBool::new(false);

pub static EXT_PROC: AtomicBool = AtomicBool::new(false);

trait TuiWidget {
    fn handle_key_event(&mut self, kv: &Key);
    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect);
    fn sync(&mut self);
}

fn probe_csi_u() {
    let mut stdout = std::io::stdout().lock();
    let _ = write!(stdout, "\x1b[?u");
    let _ = stdout.flush();
    drop(stdout);

    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut stdin = std::io::stdin().lock();
    let mut buf = [0u8; 32];
    if let Ok(n) = stdin.read(&mut buf) {
        if n > 0 && buf[..n].windows(5).any(|w| w == b"\x1b[?0u") {
            CSI_U_ENABLED.store(true, Ordering::Relaxed);
        }
    }
}

fn enable_csi_u() {
    let _ = write!(std::io::stdout(), "\x1b[=5u");
    let _ = std::io::stdout().flush();
}

fn disable_csi_u() {
    let _ = write!(std::io::stdout(), "\x1b[=0u");
    let _ = std::io::stdout().flush();
}

pub fn init() -> anyhow::Result<()> {
    agent::init()?;
    theme::Theme::load();
    raw_mode::setup()?;
    probe_csi_u();
    if CSI_U_ENABLED.load(Ordering::Relaxed) {
        enable_csi_u();
    }
    let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide);
    raw_mode::set_panic_hook();
    Ok(())
}

pub fn restore() -> anyhow::Result<()> {
    suspend_terminal(true);
    Ok(())
}

pub fn hold(on: bool) -> anyhow::Result<()> {
    if on {
        raw_mode::restore()?;
        app::FULL_RENDER.notify_one();
    } else {
        raw_mode::setup()?
    }
    Ok(())
}

pub fn suspend_terminal(permanent: bool) {
    if permanent {
        CSI_U_ENABLED.store(false, Ordering::Relaxed);
    }
    disable_csi_u();
    let _ = raw_mode::restore();
    let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Show);
    app::FULL_RENDER.notify_one();
}

pub fn resume_terminal() -> anyhow::Result<()> {
    raw_mode::setup()?;
    if CSI_U_ENABLED.load(Ordering::Relaxed) {
        enable_csi_u();
    }
    let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide);
    raw_mode::set_panic_hook();
    Ok(())
}
