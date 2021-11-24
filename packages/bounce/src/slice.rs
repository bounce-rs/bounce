use std::rc::Rc;

use crate::state::{BounceRootHandle, State, Stateful};
use crate::utils::sealed::Sealed;

/// A controlled state that is Copy-on-Write and notifies registered hooks when `prev_value != next_value`.
pub trait Slice: PartialEq + Default {
    type Action;

    /// Performs a reduce action.
    ///
    /// This always yields a new instance of [`Rc<Self>`] so it can be compared with the previous
    /// slice using [`PartialEq`].
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self>;
}

impl<T> Stateful for T
where
    T: Slice + 'static,
{
    type State = SliceState<Self>;
    type Input = T::Action;
}

pub trait CloneSlice: Slice + Clone {
    fn clone_slice(&self) -> Self {
        self.clone()
    }
}

impl<T> CloneSlice for T where T: Slice + Clone {}

pub struct SliceState<T>
where
    T: Slice,
{
    inner: Rc<T>,
}

impl<T> State<T> for SliceState<T>
where
    T: Slice + 'static,
{
    fn new(_root: BounceRootHandle) -> Self {
        SliceState {
            inner: Rc::default(),
        }
    }

    fn get(&mut self) -> Rc<T> {
        self.inner.clone()
    }

    fn set(&mut self, val: T::Action) -> bool {
        let prev_val = self.inner.clone();
        self.inner = self.inner.clone().reduce(val);

        prev_val != self.inner
    }
}

impl<T> Sealed for SliceState<T> where T: Slice + 'static {}

impl<T> Clone for SliceState<T>
where
    T: Slice,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
