use anyhow::Result;
use demotui_shared::{data::Data, frontend::InfoActOpt};

use crate::{actor::FrontEndActor, context::Ctx};

pub struct InfoAct;

impl FrontEndActor for InfoAct {
    type Options = InfoActOpt;
    fn act(ctx: &mut Ctx, opt: InfoActOpt) -> Result<Data> {
        log::debug!("Execute InfoAct");
        Ok(Data::from("Msg from FrontEndActor"))
    }
}
