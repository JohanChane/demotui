use crate::app::App;
use anyhow::Result;
use demotui_frontend::actor::FrontEndActor;
use demotui_frontend::context::Ctx;
use demotui_frontend::InfoAct;
use demotui_shared::{data::Data, frontend::op::FrontEndOp};
use log::info;

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
            FrontEndOp::Info(opt) => InfoAct::act(&mut ctx, opt),
        }
        // match cmd.layer {
        // Layer::App => self.app(cmd),
        // Layer::Mgr => self.mgr(cmd),
        // Layer::Tasks => self.tasks(cmd),
        // Layer::Spot => self.spot(cmd),
        // Layer::Pick => self.pick(cmd),
        // Layer::Input => self.input(cmd),
        // Layer::Confirm => self.confirm(cmd),
        // Layer::Help => self.help(cmd),
        // Layer::Cmp => self.cmp(cmd),
        // Layer::Which => self.which(cmd),
        // }
    }
}
