use crate::app::App;
use anyhow::Result;
use demotui_frontend::actor::FrontEndActor;
use demotui_frontend::context::Ctx;
use demotui_shared::{data::Data, frontend::op::FrontEndOp, frontend_act};

pub(super) struct Executor<'a> {
    app: &'a mut App,
}

impl<'a> Executor<'a> {
    #[inline]
    pub(super) fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    #[inline]
    pub(super) fn execute(&mut self, op: FrontEndOp) -> Result<Data> {
        let mut ctx = Ctx::new(&mut self.app.frontend)?;
        match op {
            FrontEndOp::Info(opt) => {
                // InfoAct::act(&mut ctx, opt)
                frontend_act!(InfoAct, ctx, opt)
            }
        }
    }
}
