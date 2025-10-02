use anyhow::Result;
use std::ops::{Deref, DerefMut};

use crate::backend::BackEnd;

pub struct Ctx<'a> {
    backend: &'a mut BackEnd,
}

impl Deref for Ctx<'_> {
    type Target = BackEnd;

    fn deref(&self) -> &Self::Target {
        self.backend
    }
}

impl DerefMut for Ctx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.backend
    }
}

impl<'a> Ctx<'a> {
    #[inline]
    pub fn new(backend: &'a mut BackEnd) -> Result<Self> {
        Ok(Self { backend })
    }
}
