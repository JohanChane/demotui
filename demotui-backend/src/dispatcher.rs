use anyhow::Result;
use demotui_shared::{
    backend::{self, event::BackEndEvent, op::BackEndOp},
    frontend::{InfoActOpt, event::FrontEndEvent, op::FrontEndOp},
};

use crate::actor::BackEndActor;
use crate::{AddAct, backend::BackEnd, context::Ctx};

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
    fn dispatch_call(&mut self, op: BackEndOp) -> Result<()> {
        match op {
            BackEndOp::Add(opt) => {
                let mut ctx = Ctx::new(self.backend)?;
                let result = AddAct::act(&mut ctx, opt).unwrap();
                FrontEndEvent::Call(FrontEndOp::Info(InfoActOpt {
                    msg: result.to_string(),
                }))
                .emit();
            }
        };

        Ok(())
    }
}
