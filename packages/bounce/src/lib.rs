mod atom;
mod hooks;
mod provider;
mod slice;
mod utils;

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

pub mod prelude {
    pub use crate::atom::Atom;
    pub use crate::hooks::*;
    pub use crate::slice::{CloneSlice, Slice};
}
