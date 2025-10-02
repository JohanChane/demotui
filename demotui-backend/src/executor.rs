use anyhow::Result;
use demotui_shared::frontend::InfoActOpt;
use demotui_shared::frontend_emit;
use demotui_shared::{backend::op::BackEndOp, backend_act, data::Data};

use crate::actor::BackEndActor;
use crate::{backend::BackEnd, context::Ctx};

pub(super) struct Executor<'a> {
    backend: &'a mut BackEnd,
}

impl<'a> Executor<'a> {
    #[inline]
    pub(super) fn new(backend: &'a mut BackEnd) -> Self {
        Self { backend }
    }

    #[inline]
    pub(super) fn execute(&mut self, op: BackEndOp) -> Result<Data> {
        let mut ctx = Ctx::new(self.backend)?;

        match op {
            BackEndOp::Add(opt) => {
                let result = backend_act!(AddAct, ctx, opt)?;

                if let Some(msg) = result.as_str() {
                    frontend_emit!(Call(
                        Info,
                        InfoActOpt {
                            msg: msg.to_string()
                        }
                    ));
                }

                Ok(result)
            }
        }
    }
}
