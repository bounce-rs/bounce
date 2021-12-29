use std::any::Any;
use std::rc::Rc;

/// A controlled state that is Copy-on-Write and notifies registered hooks when `prev_value != next_value`.
///
/// It can be derived for any state that implements [`Reducible`](yew::functional::Reducible) + [`PartialEq`] + [`Default`].
///
/// # Example
///
/// ```
/// use std::rc::Rc;
/// use bounce::prelude::*;
/// use yew::prelude::*;
///
/// enum CounterAction {
///     Increment,
///     Decrement,
/// }
///
/// #[derive(PartialEq, Default, Slice)]
/// struct Counter(u64);
///
/// impl Reducible for Counter {
///     type Action = CounterAction;
///
///     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
///         match action {
///             CounterAction::Increment => Self(self.0 + 1).into(),
///             CounterAction::Decrement => Self(self.0 - 1).into(),
///         }
///     }
/// }
/// ```
pub use bounce_macros::Slice;

#[doc(hidden)]
pub trait Slice: PartialEq + Default {
    type Action;

    /// Performs a reduce action.
    ///
    /// This always yields a new instance of [`Rc<Self>`] so it can be compared with the previous
    /// slice using [`PartialEq`].
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self>;

    /// Applies a notion.
    ///
    /// This always yields a new instance of [`Rc<Self>`] so it can be compared with the previous
    /// slice using [`PartialEq`].
    #[allow(unused_variables)]
    fn apply(self: Rc<Self>, notion: Rc<dyn Any>) -> Rc<Self> {
        self
    }
}

/// A trait to provide cloning on slices.
///
/// This trait provides a `self.clone_slice()` method that can be used as an alias of `(*self).clone()`
/// in reduce functions to produce a owned clone of the slice.
pub trait CloneSlice: Slice + Clone {
    /// Clones current slice.
    #[inline]
    fn clone_slice(&self) -> Self {
        self.clone()
    }
}

impl<T> CloneSlice for T where T: Slice + Clone {}
