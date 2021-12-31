#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

mod any_state;
mod atom;
mod hooks;
mod provider;
mod root_state;
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

pub use hooks::*;
pub use provider::{BounceRoot, BounceRootProps};
pub use slice::CloneSlice;
pub use with_notion::WithNotion;

pub mod prelude {
    pub use crate::atom::Atom;
    pub use crate::hooks::*;
    pub use crate::slice::{CloneSlice, Slice};
    pub use crate::with_notion::WithNotion;
}

// vendored dependencies used by macros.
#[doc(hidden)]
pub mod __vendored {
    pub use yew;
}
