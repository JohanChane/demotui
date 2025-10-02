use anyhow::Result;
use demotui_shared::data::Data;

use crate::context::Ctx;

pub trait BackEndActor {
    type Options;

    fn act(ctx: &mut Ctx, opt: Self::Options) -> Result<Data>;
}
