use anyhow::{Ok, Result};
use demotui_shared::backend::AddActOpt;

use crate::{actor::BackEndActor, context::Ctx};

pub struct AddAct;

impl BackEndActor for AddAct {
    type Options = AddActOpt;
    fn act(ctx: &mut Ctx, opt: Self::Options) -> Result<u32> {
        let sum = opt.a + opt.b;
        log::info!("sum: {}", sum);
        Ok(sum)
    }
}
