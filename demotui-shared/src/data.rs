// use std::{any::Any, borrow::Cow};
use std::{any::Any, borrow::Cow, collections::HashMap};

use anyhow::{bail, Result};

use ordered_float::OrderedFloat;
// use hashbrown::HashMap;

use crate::SStr;

// --- Data
#[derive(Debug)]
pub enum Data {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(SStr),
    List(Vec<Data>),
    Dict(HashMap<DataKey, Data>),
    Bytes(Vec<u8>),
    Any(Box<dyn Any + Send + Sync>),
}

impl Data {
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::String(s) if s == "no" => Some(false),
            Self::String(s) if s == "yes" => Some(true),
            _ => None,
        }
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn into_string(self) -> Option<SStr> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn into_dict(self) -> Option<HashMap<DataKey, Self>> {
        match self {
            Self::Dict(d) => Some(d),
            _ => None,
        }
    }

    #[inline]
    pub fn into_any<T: 'static>(self) -> Option<T> {
        match self {
            Self::Any(b) => b.downcast::<T>().ok().map(|b| *b),
            _ => None,
        }
    }

    // 修复后（正确的）：
    #[inline]
    pub fn into_any2<T: 'static>(self) -> Result<T> {
        match self {
            Self::Any(b) => match b.downcast::<T>() {
                Ok(t) => Ok(*t),
                Err(_) => bail!(
                    "Failed to downcast Data into {}",
                    std::any::type_name::<T>()
                ),
            },
            _ => bail!(
                "Data is not of type Any, cannot downcast into {}",
                std::any::type_name::<T>()
            ),
        }
    }
}

impl From<()> for Data {
    fn from(_: ()) -> Self {
        Self::Nil
    }
}

impl From<bool> for Data {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i32> for Data {
    fn from(value: i32) -> Self {
        Self::Integer(value as i64)
    }
}

impl From<u32> for Data {
    fn from(value: u32) -> Self {
        Self::Integer(value as i64)
    }
}

impl From<i64> for Data {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Data {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Data {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

impl From<SStr> for Data {
    fn from(value: SStr) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Data {
    fn from(value: &str) -> Self {
        Self::String(Cow::Owned(value.to_owned()))
    }
}

impl PartialEq<bool> for Data {
    fn eq(&self, other: &bool) -> bool {
        self.as_bool() == Some(*other)
    }
}

// --- Key
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum DataKey {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(OrderedFloat<f64>),
    String(SStr),
    Bytes(Vec<u8>),
}

impl DataKey {
    #[inline]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

impl From<usize> for DataKey {
    fn from(value: usize) -> Self {
        Self::Integer(value as i64)
    }
}

impl From<&'static str> for DataKey {
    fn from(value: &'static str) -> Self {
        Self::String(Cow::Borrowed(value))
    }
}

impl From<String> for DataKey {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

// --- Macros
macro_rules! impl_as_integer {
    ($t:ty, $name:ident) => {
        impl Data {
            #[inline]
            pub fn $name(&self) -> Option<$t> {
                match self {
                    Self::Integer(i) => <$t>::try_from(*i).ok(),
                    Self::String(s) => s.parse().ok(),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! impl_as_number {
    ($t:ty, $name:ident) => {
        impl Data {
            #[inline]
            pub fn $name(&self) -> Option<$t> {
                match self {
                    Self::Integer(i) if *i == (*i as $t as _) => Some(*i as $t),
                    Self::Number(n) => <$t>::try_from(*n).ok(),
                    Self::String(s) => s.parse().ok(),
                    _ => None,
                }
            }
        }
    };
}

impl_as_integer!(usize, as_usize);
impl_as_integer!(isize, as_isize);
impl_as_integer!(i16, as_i16);
impl_as_integer!(i32, as_i32);

impl_as_number!(f64, as_f64);
