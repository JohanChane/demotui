use anyhow::Result;
use demotui_shared::data::Data;

use crate::context::Ctx;

pub trait FrontEndActor {
    type Options;

    fn act(ctx: &mut Ctx, opt: Self::Options) -> Result<Data>;
}
