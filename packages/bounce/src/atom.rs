use std::any::Any;
use std::rc::Rc;

use crate::slice::Slice;

pub use bounce_macros::Atom;

#[doc(hidden)]
pub trait Atom: PartialEq + Default {
    #[allow(unused_variables)]
    fn apply(self: Rc<Self>, notion: Rc<dyn Any>) -> Rc<Self> {
        self
    }
}

impl<T> Slice for T
where
    T: Atom,
{
    type Action = T;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }

    fn apply(self: Rc<Self>, notion: Rc<dyn Any>) -> Rc<Self> {
        Atom::apply(self, notion)
    }
}
