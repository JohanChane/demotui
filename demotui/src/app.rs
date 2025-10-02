use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use crate::dispatcher::Dispatcher;
use anyhow::Result;
use demotui_backend::backend::{self, BackEnd};
use demotui_frontend::tui::frontend::{self, FrontEnd};
use demotui_frontend::tui::widget::root::Root;
use demotui_shared::{
    data::Data,
    frontend::event::{FrontEndEvent, NEED_RENDER},
};
use tokio::{select, time::sleep};

use crate::{signals::Signals, term::Term};

pub(crate) struct App {
    pub(crate) frontend: FrontEnd,
    pub(crate) term: Option<Term>,
    pub(crate) signals: Signals,
}

impl App {
    pub(crate) async fn serve() -> Result<()> {
        let term = Term::start()?;

        demotui_shared::init();

        BackEnd::start().await;

        let (mut rx, signals) = (FrontEndEvent::take(), Signals::start()?);

        let frontend = FrontEnd::new();
        let mut app = Self {
            frontend: frontend,
            term: Some(term),
            signals,
        };

        app.bootstrap();

        let mut events = Vec::with_capacity(50);
        let (mut timeout, mut last_render) = (None, Instant::now());
        macro_rules! drain_events {
            () => {
                for event in events.drain(..) {
                    Dispatcher::new(&mut app).dispatch(event)?;
                    if !NEED_RENDER.load(Ordering::Relaxed) {
                        continue;
                    }

                    timeout = Duration::from_millis(10).checked_sub(last_render.elapsed());
                    if timeout.is_none() {
                        app.render();
                        last_render = Instant::now();
                    }
                }
            };
        }
        loop {
            if let Some(t) = timeout.take() {
                select! {
                    _ = sleep(t) => {
                        app.render();
                        last_render = Instant::now();
                    }
                    n = rx.recv_many(&mut events, 50) => {
                        if n == 0 { break }
                        drain_events!();
                    }
                }
            } else if rx.recv_many(&mut events, 50).await != 0 {
                drain_events!();
            } else {
                break;
            }
        }

        log::info!("app exit");
        Ok(())
    }

    pub(crate) fn render(&mut self) -> Result<Data> {
        NEED_RENDER.store(false, Ordering::Relaxed);

        let Some(term) = &mut self.term else {
            return Ok(Data::Nil);
        };

        let frame = term
            .draw(|f| f.render_widget(Root::new(&self.frontend), f.area()))
            .unwrap();

        Ok(Data::Nil)
    }

    pub(crate) fn resize(&mut self) -> Result<Data> {
        if let Some(t) = &mut self.term {
            t.resize()?;
        }

        Ok(Data::Nil)
    }

    fn bootstrap(&mut self) -> Result<Data> {
        // do check

        self.render()
    }
}
