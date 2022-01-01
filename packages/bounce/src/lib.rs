#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

mod any_state;
mod atom;
mod future_notion;
mod hooks;
mod provider;
mod root_state;
mod selector;
mod slice;
mod utils;
mod with_notion;

/// A simple state that is Copy-on-Write and notifies registered hooks when `prev_value != next_value`.
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
/// See: [`use_atom`](crate::use_atom)
pub use atom::Atom;

/// A reducer-based state that is Copy-on-Write and notifies registered hooks when `prev_value != next_value`.
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
/// See: [`use_slice`](crate::use_slice)
pub use slice::Slice;

/// A future-based notion that notifies states when it begins and finishes.
///
/// A future notion accepts a signle argument as input and returns an output.
///
/// It can optionally accept a `states` parameter which has a type of [`BounceStates`] that can be
/// used to access bounce states when being run.
///
/// `Fn(Rc<I>) -> impl Future<Output = Rc<O>>` or `Fn(BounceState, Rc<I>) -> impl Future<Output = Rc<O>>`.
///
/// Both `Input` and `Output` must be Rc'ed.
///
/// # Example
///
/// ```
/// use std::rc::Rc;
/// use bounce::prelude::*;
/// use yew::prelude::*;
///
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// #[future_notion(FetchData)]
/// async fn fetch_user(id: Rc<u64>) -> Rc<User> {
///     // fetch user
///
///     User { id: *id, name: "John Smith".into() }.into()
/// }
/// ```
/// See: [`use_future_notion_runner`](crate::use_future_notion_runner)
pub use bounce_macros::future_notion;

pub use atom::CloneAtom;
pub use future_notion::{Deferred, FutureNotion};
#[doc(inline)]
pub use hooks::*;
pub use provider::{BounceRoot, BounceRootProps};
pub use root_state::BounceStates;
pub use slice::CloneSlice;
pub use with_notion::WithNotion;

pub mod prelude {
    //! Default Bounce exports.

    pub use crate::future_notion;
    pub use crate::hooks::*;
    pub use crate::BounceStates;
    pub use crate::WithNotion;
    pub use crate::{Atom, CloneAtom};
    pub use crate::{CloneSlice, Slice};
    pub use crate::{Deferred, FutureNotion};
}

// vendored dependencies used by macros.
#[doc(hidden)]
pub mod __vendored {
    pub use futures;
    pub use yew;
}
