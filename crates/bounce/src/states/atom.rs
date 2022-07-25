use std::any::{Any, TypeId};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use super::slice::{use_slice, use_slice_dispatch, use_slice_value, Slice, UseSliceHandle};

pub use bounce_macros::Atom;
use yew::prelude::*;

#[doc(hidden)]
pub trait Atom: PartialEq + Default {
    #[allow(unused_variables)]
    fn apply(self: Rc<Self>, notion: Rc<dyn Any>) -> Rc<Self> {
        self
    }

    fn notion_ids(&self) -> Vec<TypeId>;

    fn changed(self: Rc<Self>) {}
}

/// A trait to provide cloning on atoms.
///
/// This trait provides a `self.clone_atom()` method that can be used as an alias of `(*self).clone()`
/// in apply functions to produce a owned clone of the atom.
pub trait CloneAtom: Atom + Clone {
    /// Clones current atom.
    #[inline]
    fn clone_atom(&self) -> Self {
        self.clone()
    }
}

impl<T> CloneAtom for T where T: Atom + Clone {}

#[derive(PartialEq, Default)]
pub(crate) struct AtomSlice<T>
where
    T: Atom,
{
    pub inner: Rc<T>,
}

impl<T> Slice for AtomSlice<T>
where
    T: Atom,
{
    type Action = T;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        Self {
            inner: action.into(),
        }
        .into()
    }

    fn apply(self: Rc<Self>, notion: Rc<dyn Any>) -> Rc<Self> {
        Self {
            inner: self.inner.clone().apply(notion),
        }
        .into()
    }

    fn notion_ids(&self) -> Vec<TypeId> {
        self.inner.notion_ids()
    }

    fn changed(self: Rc<Self>) {
        self.inner.clone().changed();
    }
}

/// A handle returned by [`use_atom`].
///
/// This type dereferences to `T` and has a `set` method to set value for current state.
pub struct UseAtomHandle<T>
where
    T: Atom,
{
    inner: UseSliceHandle<AtomSlice<T>>,
}

impl<T> UseAtomHandle<T>
where
    T: Atom + 'static,
{
    /// Sets the value of current atom.
    pub fn set(&self, val: T) {
        self.inner.dispatch(val)
    }
}

impl<T> Deref for UseAtomHandle<T>
where
    T: Atom,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &(*self.inner).inner
    }
}

impl<T> Clone for UseAtomHandle<T>
where
    T: Atom,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> fmt::Debug for UseAtomHandle<T>
where
    T: Atom + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseAtomHandle")
            .field("inner", &*self)
            .finish()
    }
}

/// A hook to connect to an [`Atom`](macro@crate::Atom).
///
/// Returns a [`UseAtomHandle<T>`].
///
/// # Example
///
/// ```
/// # use std::fmt;
/// # use bounce::prelude::*;
/// # use yew::prelude::*;
/// # use web_sys::HtmlInputElement;
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
///
/// impl From<String> for Username {
///     fn from(s: String) -> Self {
///         Self { inner: s }
///     }
/// }
///
/// impl fmt::Display for Username {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.inner)
///     }
/// }
///
/// #[function_component(Setter)]
/// fn setter() -> Html {
///     let username = use_atom::<Username>();
///
///     let on_text_input = {
///         let username = username.clone();
///
///         Callback::from(move |e: InputEvent| {
///             let input: HtmlInputElement = e.target_unchecked_into();
///
///             username.set(input.value().into());
///         })
///     };
///
///     html! {
///         <div>
///             <input type_="text" oninput={on_text_input} value={username.to_string()} />
///         </div>
///     }
/// }
/// ```
#[hook]
pub fn use_atom<T>() -> UseAtomHandle<T>
where
    T: Atom + 'static,
{
    let inner = use_slice::<AtomSlice<T>>();

    UseAtomHandle { inner }
}

/// A hook to produce a setter function for an [`Atom`](macro@crate::Atom).
///
/// Returns a `Rc<dyn Fn(T)>`.
///
/// This hook will return a setter function that will not change across the entire lifetime of the
/// component.
///
/// ```
/// # use bounce::prelude::*;
/// # use std::fmt;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # #[derive(PartialEq, Atom)]
/// # struct Username {
/// #     inner: String,
/// # }
/// #
/// # impl From<&str> for Username {
/// #     fn from(s: &str) -> Self {
/// #         Self { inner: s.into() }
/// #     }
/// # }
/// #
/// # impl Default for Username {
/// #     fn default() -> Self {
/// #         Self {
/// #             inner: "Jane Doe".into(),
/// #         }
/// #     }
/// # }
/// #
/// # impl fmt::Display for Username {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "{}", self.inner)
/// #     }
/// # }
/// #
/// # #[function_component(Setter)]
/// # fn setter() -> Html {
/// let set_username = use_atom_setter::<Username>();
/// set_username("John Smith".into());
/// # Html::default()
/// # }
/// ```
#[hook]
pub fn use_atom_setter<T>() -> Rc<dyn Fn(T)>
where
    T: Atom + 'static,
{
    use_slice_dispatch::<AtomSlice<T>>()
}

/// A read-only hook to connect to the value of an [`Atom`](macro@crate::Atom).
///
/// Returns `Rc<T>`.
///
/// # Example
///
/// ```
/// # use std::fmt;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # #[derive(PartialEq, Atom)]
/// # struct Username {
/// #     inner: String,
/// # }
/// #
/// # impl Default for Username {
/// #     fn default() -> Self {
/// #         Self {
/// #             inner: "Jane Doe".into(),
/// #         }
/// #     }
/// # }
/// #
/// # impl fmt::Display for Username {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "{}", self.inner)
/// #     }
/// # }
/// #
/// #[function_component(Reader)]
/// fn reader() -> Html {
///     let username = use_atom_value::<Username>();
///
///     html! { <div>{"Hello, "}{username}</div> }
/// }
/// ```
#[hook]
pub fn use_atom_value<T>() -> Rc<T>
where
    T: Atom + 'static,
{
    use_slice_value::<AtomSlice<T>>().inner.clone()
}
