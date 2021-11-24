use std::rc::Rc;

use crate::state::{BounceRootHandle, State, Stateful};
use crate::utils::sealed::Sealed;
use crate::{Atom, Slice};

pub struct SelectorGetHandle<T>
where
    T: Selector + 'static,
{
    inner: SelectorState<T>,
}

impl<S> SelectorGetHandle<S>
where
    S: Selector + 'static,
{
    pub fn get<T>(&self) -> Rc<T>
    where
        T: Slice + 'static,
    {
        todo!()
    }
}

pub struct SelectorSetHandle<T>
where
    T: Selector + 'static,
{
    inner: SelectorState<T>,
}

impl<S> SelectorSetHandle<S>
where
    S: Selector + 'static,
{
    pub fn get<T>(&self) -> Rc<T>
    where
        T: Slice + 'static,
    {
        todo!()
    }

    pub fn set<T>(&self, val: T)
    where
        T: Atom + 'static,
    {
        todo!()
    }
}

pub trait Selector: Sized {
    type Input;

    fn get(handle: SelectorGetHandle<Self>) -> Rc<Self>;
    fn set(val: Self::Input, handle: SelectorSetHandle<Self>) {
        unimplemented!("You need to implement reduce before using it.")
    }
}

impl<T> Stateful for T
where
    T: Selector + 'static,
{
    type State = SelectorState<Self>;
    type Input = <T as Selector>::Input;
}

pub struct SelectorState<T>
where
    T: Selector + 'static,
{
    inner: Option<T>,
    root: BounceRootHandle,
}

impl<T> Sealed for SelectorState<T> where T: Selector + 'static {}

impl<T> State<T> for SelectorState<T>
where
    T: Selector + 'static,
{
    fn new(root: BounceRootHandle) -> Self {
        Self { inner: None, root }
    }
}
