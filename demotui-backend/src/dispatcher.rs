use anyhow::Result;
use demotui_shared::{
    backend::{self, event::BackEndEvent, op::BackEndOp},
    data::Data,
    frontend::{InfoActOpt, event::FrontEndEvent, op::FrontEndOp},
};
use demotui_shared::{backend_act, frontend_emit};

use crate::{AddAct, backend::BackEnd, context::Ctx};
use crate::{actor::BackEndActor, executor::Executor};

pub(super) struct Dispatcher<'a> {
    backend: &'a mut BackEnd,
}

impl<'a> Dispatcher<'a> {
    #[inline]
    pub(super) fn new(backend: &'a mut BackEnd) -> Self {
        Self { backend }
    }

    #[inline]
    pub(super) fn dispatch(&mut self, event: BackEndEvent) -> Result<()> {
        _ = match event {
            BackEndEvent::Call(op) => self.dispatch_call(op)?,
        };

        Ok(())
    }

    #[inline]
    fn dispatch_call(&mut self, op: BackEndOp) -> Result<Data> {
        Executor::new(self.backend).execute(op)
    }
}
