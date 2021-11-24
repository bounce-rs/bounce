use std::rc::Rc;

use crate::provider::BounceRootState;
use crate::utils::sealed::Sealed;

pub trait Stateful: Sized {
    type State: State<Self>;
    type Input;
}

pub struct BounceRootHandle {
    _inner: BounceRootState,
}

impl From<BounceRootState> for BounceRootHandle {
    fn from(m: BounceRootState) -> Self {
        Self { _inner: m }
    }
}

pub trait State<T>: Sealed + Clone
where
    T: Stateful,
{
    fn new(root: BounceRootHandle) -> Self;
    fn get(&mut self) -> Rc<T>;
    fn set(&mut self, val: T::Input) -> bool;
}
