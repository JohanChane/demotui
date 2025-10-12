use std::{os::unix::fs::OpenOptionsExt, sync::atomic::Ordering};

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
    frontend_emit,
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
            FrontEndEvent::Seq(ops) => self.dispatch_seq(ops),
            FrontEndEvent::Render => self.app.render(),
            FrontEndEvent::Resize => self.app.resize(),
            FrontEndEvent::Quit(opt) => self.app.quit(opt),
        };

        Ok(())
    }

    pub fn dispatch_call(&self, op: FrontEndOp) -> Result<Data> {
        match op {
            FrontEndOp::Info(opt) => {
                log::debug!("data: {}", opt.msg);
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

    pub fn dispatch_seq(&self, mut ops: Vec<FrontEndOp>) -> Result<Data> {
        if let Some(last_op) = ops.pop() {
            self.dispatch_call(last_op);
        }
        if !ops.is_empty() {
            frontend_emit!(Seq(ops));
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
