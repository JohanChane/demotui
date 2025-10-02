use std::sync::atomic::Ordering;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use demotui_backend::backend;
use demotui_shared::{
    backend::{event::BackEndEvent, op::BackEndOp, AddActOpt},
    backend_emit,
    data::Data,
    frontend::{
        event::{FrontEndEvent, NEED_RENDER},
        op::FrontEndOp,
    },
};

use crate::app::App;

pub struct Dispatcher<'a> {
    app: &'a mut App,
}

impl<'a> Dispatcher<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    pub fn dispatch(&mut self, event: FrontEndEvent) -> Result<()> {
        _ = match event {
            FrontEndEvent::Call(op) => self.dispatch_call(op),
            FrontEndEvent::Key(key_event) => self.dispatch_key(key_event),
            FrontEndEvent::Render => self.app.render(),
            FrontEndEvent::Resize => self.app.resize(),
        };

        Ok(())
    }

    pub fn dispatch_call(&self, op: FrontEndOp) -> Result<Data> {
        match op {
            FrontEndOp::Info(opt) => {
                log::info!("data: {}", opt.msg);
            }
        };

        Ok(Data::Nil)
    }

    pub fn dispatch_key(&self, key_event: KeyEvent) -> Result<Data> {
        match key_event.code {
            KeyCode::Char('a') => {
                // BackEndEvent::Call(BackEndOp::Add(AddActOpt { a: 1, b: 2 })).emit()
                backend_emit!(Call(Add, AddActOpt { a: 1, b: 2 }));
            }
            _ => {}
        }
        Ok(Data::Nil)
    }

    #[inline]
    fn dispatch_render(&mut self) -> Result<()> {
        Ok(NEED_RENDER.store(true, Ordering::Relaxed))
    }

    // #[inline]
    // pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

    // #[inline]
    // pub(super) fn dispatch(&mut self, event: Event) -> Result {
    // 	// FIXME: handle errors
    //     _ = match event {
    // 		Event::Call(op) => self.dispatch_call(op),
    //     }
    // }

    // #[inline]
    // fn dispatch_call(&mut self, op: BackEndOp) -> Result {
    //     match op {
    //         BackEndOp::Update(data) => {act!(FrontEndEvent::call(xxx, xxx))},
    //     }
    //  }
    // }
}
