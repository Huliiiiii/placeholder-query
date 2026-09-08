#[doc(hidden)]
pub mod mode;
#[doc(hidden)]
pub mod record;
mod types;

pub use types::*;

use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

use placeholder_query_core::types::ParamId;

#[derive_where::derive_where(Clone, Copy, Debug)]
pub struct Param<T> {
    pub(crate) id: ParamId,
    _type: PhantomData<fn() -> T>,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputScope(u64);

impl InputScope {
    pub(crate) fn new() -> Self {
        static NEXT_SCOPE: AtomicU64 = AtomicU64::new(0);

        Self(NEXT_SCOPE.fetch_add(1, Ordering::Relaxed))
    }

    pub fn param<T>(self, index: u16) -> Param<T> {
        Param {
            id: ParamId::new(self.0, index),
            _type: PhantomData,
        }
    }
}

impl PartialEq<u64> for InputScope {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

pub trait QueryInput {
    type Params;

    #[doc(hidden)]
    fn params(scope: InputScope) -> Self::Params;

    #[doc(hidden)]
    type Values<'a>
    where
        Self: 'a;

    #[doc(hidden)]
    fn values(&self) -> Self::Values<'_>;
}

impl QueryInput for () {
    type Params = ();
    type Values<'a> = ();

    fn params(_: InputScope) -> Self::Params {}

    fn values(&self) -> Self::Values<'_> {}
}
