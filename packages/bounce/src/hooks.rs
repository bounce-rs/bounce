use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::atom::Atom;
use crate::provider::BounceRootState;
use crate::slice::Slice;

/// A handle returned by [`use_slice`].
///
/// This type dereferences to `T` and has a `dispatch` method to dispatch actions.
pub struct UseSliceHandle<T>
where
    T: Slice,
{
    inner: Rc<T>,
    root: BounceRootState,
}

impl<T> UseSliceHandle<T>
where
    T: Slice + 'static,
{
    /// Dispatches `Action`.
    pub fn dispatch(&self, action: T::Action) {
        self.root.dispatch_action::<T>(action);
    }
}

impl<T> Deref for UseSliceHandle<T>
where
    T: Slice,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Clone for UseSliceHandle<T>
where
    T: Slice,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            root: self.root.clone(),
        }
    }
}

impl<T> fmt::Debug for UseSliceHandle<T>
where
    T: Slice + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseSliceHandle")
            .field("inner", &self.inner)
            .finish()
    }
}

/// A hook to connect to a [`Slice`].
///
/// Returns a [`UseSliceHandle<T>`].
///
/// # Example
///
/// ```
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// #
/// enum CounterAction {
///     Increment,
///     Decrement,
/// }
///
/// #[derive(PartialEq, Default)]
/// struct Counter(u64);
///
/// impl Slice for Counter {
///     type Action = CounterAction;
///
///     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
///         match action {
///             CounterAction::Increment => Self(self.0 + 1).into(),
///             CounterAction::Decrement => Self(self.0 - 1).into(),
///         }
///     }
/// }
///
/// #[function_component(CounterComp)]
/// fn counter_comp() -> Html {
///     let ctr = use_slice::<Counter>();
///
///     let inc = {
///         let ctr = ctr.clone();
///         Callback::from(move |_| {ctr.dispatch(CounterAction::Increment);})
///     };
///     let dec = {
///         let ctr = ctr.clone();
///         Callback::from(move |_| {ctr.dispatch(CounterAction::Decrement);})
///     };;
///
///     html! {
///         <div>
///             <div>{"Current Counter: "}{ctr.0}</div>
///             <button onclick={inc}>{"Increase"}</button>
///             <button onclick={dec}>{"Decrease"}</button>
///         </div>
///     }
/// }
/// ```
pub fn use_slice<T>() -> UseSliceHandle<T>
where
    T: Slice + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    let val = {
        let root = root.clone();
        use_state_eq(move || root.get::<T>())
    };

    {
        let val = val.clone();
        let root = root.clone();
        use_effect_with_deps(
            move |root| {
                let listener = root.listen::<T, _>(move |root| {
                    val.set(root.get::<T>());
                });

                move || {
                    std::mem::drop(listener);
                }
            },
            root,
        );
    }

    let val = (*val).clone();

    UseSliceHandle { inner: val, root }
}

/// A hook to produce a dispatch function for a [`Slice`].
///
/// Returns a `Rc<dyn Fn(T::Action)>`.
///
/// This hook will return a dispatch function that will not change across the entire lifetime of the
/// component.
///
/// # Example
///
/// ```
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// #
/// # enum CounterAction {
/// #     Increment,
/// #     Decrement,
/// # }
/// #
/// # #[derive(PartialEq, Default)]
/// # struct Counter(u64);
/// #
/// # impl Slice for Counter {
/// #     type Action = CounterAction;
/// #
/// #     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
/// #         match action {
/// #             CounterAction::Increment => Self(self.0 + 1).into(),
/// #             CounterAction::Decrement => Self(self.0 - 1).into(),
/// #         }
/// #     }
/// # }
/// #
/// #[function_component(CounterComp)]
/// fn counter_comp() -> Html {
///     let dispatch_ctr = use_dispatch_slice_action::<Counter>();
///
///     let inc = {
///         let dispatch_ctr = dispatch_ctr.clone();
///         Callback::from(move |_| {dispatch_ctr(CounterAction::Increment);})
///     };
///     let dec = {
///         let dispatch_ctr = dispatch_ctr.clone();
///         Callback::from(move |_| {dispatch_ctr(CounterAction::Decrement);})
///     };;
///
///     html! {
///         <div>
///             <button onclick={inc}>{"Increase"}</button>
///             <button onclick={dec}>{"Decrease"}</button>
///         </div>
///     }
/// }
/// ```
pub fn use_dispatch_slice_action<T>() -> Rc<dyn Fn(T::Action)>
where
    T: Slice + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    // Recreate the dispatch function in case root has changed.
    Rc::new(move |action: T::Action| {
        root.dispatch_action::<T>(action);
    })
}

/// A read-only hook to connect to the value of a [`Slice`].
///
/// Returns `Rc<T>`.
///
/// # Example
///
/// ```
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// #
/// # enum CounterAction {
/// #     Increment,
/// #     Decrement,
/// # }
/// #
/// # #[derive(PartialEq, Default)]
/// # struct Counter(u64);
/// #
/// # impl Slice for Counter {
/// #     type Action = CounterAction;
/// #
/// #     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
/// #         match action {
/// #             CounterAction::Increment => Self(self.0 + 1).into(),
/// #             CounterAction::Decrement => Self(self.0 - 1).into(),
/// #         }
/// #     }
/// # }
/// #
/// #[function_component(CounterComp)]
/// fn counter_comp() -> Html {
///     let ctr = use_slice_value::<Counter>();
///
///     html! {
///         <div>
///             <div>{"Current Counter: "}{ctr.0}</div>
///         </div>
///     }
/// }
/// ```
pub fn use_slice_value<T>() -> Rc<T>
where
    T: Slice + 'static,
{
    use_slice::<T>().inner
}

/// A handle returned by [`use_atom`].
///
/// This type dereferences to `T` and has a `set` method to set value for current state.
pub struct UseAtomHandle<T>
where
    T: Atom,
{
    inner: UseSliceHandle<T>,
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
        &self.inner
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
            .field("inner", &self.inner)
            .finish()
    }
}

/// A hook to connect to an `Atom`.
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
pub fn use_atom<T>() -> UseAtomHandle<T>
where
    T: Atom + 'static,
{
    let inner = use_slice::<T>();

    UseAtomHandle { inner }
}

/// A hook to produce a setter function for a `Atom`.
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
/// let set_username = use_set_atom_value::<Username>();
/// set_username("John Smith".into());
/// # Html::default()
/// # }
/// ```
pub fn use_set_atom_value<T>() -> Rc<dyn Fn(T)>
where
    T: Atom + 'static,
{
    use_dispatch_slice_action::<T>()
}

/// A read-only hook to connect to the value of an `Atom`.
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
pub fn use_atom_value<T>() -> Rc<T>
where
    T: Slice + 'static,
{
    use_slice_value::<T>()
}
