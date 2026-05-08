use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
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

pub fn init() -> anyhow::Result<()> {
    agent::init()?;
    theme::Theme::load();
    raw_mode::setup()?;
    probe_csi_u();
    if CSI_U_ENABLED.load(Ordering::Relaxed) {
        let _ = crossterm::execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS,
            )
        );
    }
    raw_mode::set_panic_hook();
    Ok(())
}

pub fn restore() -> anyhow::Result<()> {
    if CSI_U_ENABLED.swap(false, Ordering::Relaxed) {
        use crossterm::event::PopKeyboardEnhancementFlags;
        let _ = crossterm::execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
    }
    raw_mode::restore()?;
    Ok(())
}

/// Leave RawMode and get back to main screen
pub fn hold(on: bool) -> anyhow::Result<()> {
    if on {
        raw_mode::restore()?;
        // tell ratatui to re-render
        app::FULL_RENDER.notify_one();
    } else {
        raw_mode::setup()?
    }
    Ok(())
}
