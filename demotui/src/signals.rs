use std::mem::swap;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use demotui_shared::{frontend::event::FrontEndEvent, frontend_emit};
use futures_lite::StreamExt;
#[cfg(unix)]
use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP};
use tokio::{select, task::spawn_blocking};

pub(super) struct Signals {}

impl Signals {
    pub(super) fn start() -> Result<Self> {
        Self::spawn();
        Ok(Self {})
    }

    fn handle_term(event: CrosstermEvent) {
        match event {
            CrosstermEvent::Key(
                key @ KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                },
            ) => {
                // FrontEndEvent::Key(key).emit();
                frontend_emit!(Key(key));
            }
            CrosstermEvent::Resize(..) => {
                frontend_emit!(Resize);
            }
            _ => {}
        }
    }

    #[cfg(unix)]
    fn handle_sys(n: libc::c_int) -> bool {
        use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGSTOP, SIGTERM, SIGTSTP};

        match n {
            SIGINT => {
                /* ignored */
                log::debug!("caught SIGINT");
            }
            SIGQUIT | SIGHUP | SIGTERM => {
                FrontEndEvent::Quit(Default::default()).emit();
                log::debug!("SIGQUIT | SIGHUP | SIGTERM");
                return false;
            }
            _ => {}
        }
        true
    }

    pub fn spawn() -> Result<()> {
        #[cfg(unix)]
        let mut sys = signal_hook_tokio::Signals::new([
            // Interrupt signals (Ctrl-C, Ctrl-\)
            SIGINT, SIGQUIT, //
            // Hangup signal (Terminal closed)
            SIGHUP, //
            // Termination signal (kill)
            SIGTERM, //
            // Job control signals (Ctrl-Z, fg/bg)
            SIGTSTP, SIGCONT,
        ])?;

        let mut stream = Some(event::EventStream::new());
        tokio::spawn(async move {
            loop {
                if let Some(s) = &mut stream {
                    select! {
                        biased;
                        Some(n) = sys.next() => if !Self::handle_sys(n) { return },
                        Some(Ok(e)) = s.next() => Self::handle_term(e),
                    }
                } else {
                    select! {
                        biased;
                        Some(n) = sys.next() => if !Self::handle_sys(n) { return },
                    }
                }
            }
        });

        Ok(())
    }
}
