use std::io::Write;

use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend, buffer::Buffer, layout::Rect, CompletedFrame, Frame, Terminal,
};
use tokio::time::Sleep;

type TtyWriterType = std::io::Stdout;
pub(crate) struct Term {
    inner: Terminal<CrosstermBackend<TtyWriterType>>,
    last_area: Rect,
    last_buffer: Buffer,
}

impl Term {
    pub(crate) fn start() -> Result<(Self)> {
        Self::setup()?;
        Self::set_panic_hook();

        let mut term = Self {
            inner: Terminal::new(CrosstermBackend::new(std::io::stdout()))?,
            last_area: Default::default(),
            last_buffer: Default::default(),
        };

        Ok((term))
    }

    pub(crate) fn draw(
        &mut self,
        f: impl FnOnce(&mut Frame),
    ) -> std::io::Result<CompletedFrame<'_>> {
        let last = self.inner.draw(f)?;

        self.last_area = last.area;
        self.last_buffer = last.buffer.clone();
        Ok(last)
    }

    /// Enables raw mode and sets up the terminal for the application.
    pub(crate) fn setup() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)
    }

    /// Disables raw mode and restores the terminal to its original state.
    pub(crate) fn restore() -> Result<(), std::io::Error> {
        disable_raw_mode()?;
        execute!(
            std::io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )
    }

    /// make terminal restorable after panic
    pub(crate) fn set_panic_hook() {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            let _ = Self::restore();
            original_hook(panic);
        }));
    }

    pub(crate) fn resize(&mut self) -> Result<()> {
        self.inner.autoresize()?;
        Ok(())
    }

    // TODO
    pub(super) fn goodbye(f: impl FnOnce() -> i32) -> ! {
        Self::restore();

        log::debug!("goodbye");
        std::process::exit(f());
    }
}
