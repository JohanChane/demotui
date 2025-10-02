use anyhow::{Ok, Result};
use demotui_shared::{backend::AddActOpt, data::Data};

use crate::{actor::BackEndActor, context::Ctx};

pub struct AddAct;

impl BackEndActor for AddAct {
    type Options = AddActOpt;
    fn act(ctx: &mut Ctx, opt: Self::Options) -> Result<Data> {
        let sum = opt.a + opt.b;
        log::debug!("sum: {}", sum);
        Ok(Data::from(sum))
    }
}
