use std::any::Any;
use std::rc::Rc;

use crate::slice::Slice;

/// A simple controlled state that is Copy-on-Write and notifies registered hooks when `prev_value != next_value`.
///
/// It can be derived for any state that implements [`PartialEq`] + [`Default`].
///
/// # Example
///
/// ```
/// use std::rc::Rc;
/// use bounce::prelude::*;
/// use yew::prelude::*;
///
/// #[derive(PartialEq, Atom)]
/// struct Username {
///     inner: String,
/// }
///
/// impl Default for Username {
///     fn default() -> Self {
///         Self {
///             inner: "Jane Doe".into(),
///         }
///     }
/// }
/// ```
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
