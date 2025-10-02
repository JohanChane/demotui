use anyhow::Result;
use demotui_shared::frontend;
use std::ops::{Deref, DerefMut};

use crate::tui::frontend::FrontEnd;

pub struct Ctx<'a> {
    pub frontend: &'a mut FrontEnd,
}

impl Deref for Ctx<'_> {
    type Target = FrontEnd;

    fn deref(&self) -> &Self::Target {
        self.frontend
    }
}

impl DerefMut for Ctx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.frontend
    }
}

impl<'a> Ctx<'a> {
    #[inline]
    pub fn new(frontend: &'a mut FrontEnd) -> Result<Self> {
        Ok(Self { frontend })
    }
}
