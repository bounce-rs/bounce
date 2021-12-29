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
mod slice;
mod utils;
mod with_notion;

/// A Simple State.
///
/// It can be derived on any type that is `PartialEq` + `Default`.
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// #
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
///
/// See: [`use_atom`](crate::use_atom)
pub use atom::Atom;
pub use hooks::*;
pub use provider::{BounceRoot, BounceRootProps};
pub use slice::{CloneSlice, Slice};
pub use with_notion::WithNotion;

pub mod prelude {
    pub use crate::atom::Atom;
    pub use crate::hooks::*;
    pub use crate::slice::{CloneSlice, Slice};
    pub use crate::with_notion::WithNotion;
}

#[doc(hidden)]
pub mod __vendored {
    pub use yew;
}
