use anyhow::Result;

use crate::context::Ctx;

pub trait BackEndActor {
    type Options;

    fn act(ctx: &mut Ctx, opt: Self::Options) -> Result<u32>;
}
