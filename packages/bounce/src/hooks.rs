use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::atom::{Atom, AtomSlice};
use crate::future_notion::{Deferred, FutureNotion};
use crate::input_selector::{InputSelector, InputSelectorsState};
use crate::root_state::BounceRootState;
use crate::selector::{Selector, UnitSelector};
use crate::slice::Slice;
use crate::slice::SliceState;
use crate::utils::RcTrait;

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
        self.root.get_state::<SliceState<T>>().dispatch(action);
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

/// A hook to connect to a `Slice`.
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
        use_state_eq(move || root.get_state::<SliceState<T>>().get())
    };

    {
        let val = val.clone();
        let root = root.clone();
        use_effect_with_deps(
            move |root| {
                let listener = root
                    .get_state::<SliceState<T>>()
                    .listen(Rc::new(Callback::from(move |m| {
                        val.set(m);
                    })));

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

/// A hook to produce a dispatch function for a `Slice`.
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
/// # #[derive(PartialEq, Default, Slice)]
/// # struct Counter(u64);
/// #
/// # impl Reducible for Counter {
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
///     let dispatch_ctr = use_slice_dispatch::<Counter>();
///
///     let inc = {
///         let dispatch_ctr = dispatch_ctr.clone();
///         Callback::from(move |_| {dispatch_ctr(CounterAction::Increment);})
///     };
///     let dec = {
///         let dispatch_ctr = dispatch_ctr.clone();
///         Callback::from(move |_| {dispatch_ctr(CounterAction::Decrement);})
///     };
///
///     html! {
///         <div>
///             <button onclick={inc}>{"Increase"}</button>
///             <button onclick={dec}>{"Decrease"}</button>
///         </div>
///     }
/// }
/// ```
pub fn use_slice_dispatch<T>() -> Rc<dyn Fn(T::Action)>
where
    T: Slice + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    // Recreate the dispatch function in case root has changed.
    Rc::new(move |action: T::Action| {
        root.get_state::<SliceState<T>>().dispatch(action);
    })
}

/// A read-only hook to connect to the value of a `Slice`.
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
/// # #[derive(PartialEq, Default, Slice)]
/// # struct Counter(u64);
/// #
/// # impl Reducible for Counter {
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
    let inner = use_slice::<AtomSlice<T>>();

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
/// let set_username = use_atom_setter::<Username>();
/// set_username("John Smith".into());
/// # Html::default()
/// # }
/// ```
pub fn use_atom_setter<T>() -> Rc<dyn Fn(T)>
where
    T: Atom + 'static,
{
    use_slice_dispatch::<AtomSlice<T>>()
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
    T: Atom + 'static,
{
    use_slice_value::<AtomSlice<T>>().inner.clone()
}

/// A hook to create a function that applies a `Notion`.
///
/// A `Notion` is an action that can be dispatched to any state that accepts the dispatched notion.
///
/// Any type that is `'static` can be dispatched as a notion.
///
/// Returns `Rc<dyn Fn(T)>`.
///
/// # Note
///
/// When states receives a notion, it will be wrapped in an `Rc<T>`.
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// # use std::fmt;
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// pub struct Reset;
///
/// #[derive(PartialEq, Atom)]
/// #[with_notion(Reset)] // A #[with_notion(Notion)] needs to be denoted for the notion.
/// struct Username {
///     inner: String,
/// }
///
/// // A WithNotion<T> is required for each notion denoted in the #[with_notion] attribute.
/// impl WithNotion<Reset> for Username {
///     fn apply(self: Rc<Self>, _notion: Rc<Reset>) -> Rc<Self> {
///         Self::default().into()
///     }
/// }
///
/// // second state
/// #[derive(PartialEq, Atom, Default)]
/// #[with_notion(Reset)]
/// struct Session {
///     token: Option<String>,
/// }
///
/// impl WithNotion<Reset> for Session {
///     fn apply(self: Rc<Self>, _notion: Rc<Reset>) -> Rc<Self> {
///         Self::default().into()
///     }
/// }
/// #
/// # impl Default for Username {
/// #     fn default() -> Self {
/// #         Self {
/// #             inner: "Jane Doe".into(),
/// #         }
/// #     }
/// # }
/// #
/// # #[function_component(Setter)]
/// # fn setter() -> Html {
/// let reset_everything = use_notion_applier::<Reset>();
/// reset_everything(Reset);
/// # Html::default()
/// # }
/// ```
pub fn use_notion_applier<T>() -> Rc<dyn Fn(T)>
where
    T: 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    // Recreate the dispatch function in case root has changed.
    Rc::new(move |notion: T| {
        root.apply_notion(Rc::new(notion) as Rc<dyn Any>);
    })
}

/// A hook to create a function that when called, runs a `FutureNotion` with provided input.
///
/// A `FutureNotion` is created by applying a `#[future_notion(NotionName)]` attribute to an async function.
///
/// When a future notion is run, it will be applied twice with a notion type [`Deferred<T>`]. The
/// first time is before it starts with a variant `Pending` and the second time is when it
/// completes with variant `Complete`.
///
/// # Note
///
/// If you are trying to interact with a backend API, it is recommended to use the [Query](crate::query) API instead.
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// # use std::fmt;
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
///
/// #[derive(PartialEq)]
/// struct User {
///     id: u64,
///     username: String,
/// }
///
/// #[future_notion(FetchUser)]
/// async fn fetch_user(id: Rc<u64>) -> Rc<User> {
///     // fetch user here...
///
///     User { id: *id, username: "username".into() }.into()
/// }
///
/// #[derive(PartialEq, Default, Atom)]
/// #[with_notion(Deferred<FetchUser>)]  // A future notion with type `T` will be applied as `Deferred<T>`.
/// struct UserState {
///     inner: Option<Rc<User>>,
/// }
///
/// // Each time a future notion is run, it will be applied twice.
/// impl WithNotion<Deferred<FetchUser>> for UserState {
///     fn apply(self: Rc<Self>, notion: Rc<Deferred<FetchUser>>) -> Rc<Self> {
///         match notion.output() {
///             Some(m) => Self { inner: Some(m) }.into(),
///             None => self,
///         }
///     }
/// }
///
/// # #[function_component(FetchUserComp)]
/// # fn fetch_user_comp() -> Html {
/// let load_user = use_future_notion_runner::<FetchUser>();
/// load_user(1.into());
/// # Html::default()
/// # }
/// ```
pub fn use_future_notion_runner<T>() -> Rc<dyn Fn(T::Input)>
where
    T: FutureNotion + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    Rc::new(move |input: T::Input| {
        let root = root.clone();
        let input = input.clone_rc();

        spawn_local(async move {
            root.apply_notion(Rc::new(Deferred::<T>::Pending {
                input: input.clone_rc(),
            }) as Rc<dyn Any>);

            let states = root.states();

            // send the listeners in to be destroyed.
            let listeners = Rc::new(RefCell::new(None));
            let listener_run = Rc::new(AtomicBool::new(false));

            {
                let listener_run = listener_run.clone();
                let listeners = listeners.clone();
                let root = root.clone();
                let input = input.clone_rc();
                states.add_listener_callback(Rc::new(Callback::from(move |_| {
                    // There's a chance that the listeners might be called during the time while the future
                    // notion is running and there will be nothing to drop.
                    let listeners = listeners.borrow_mut().take();
                    let last_listener_run = listener_run.swap(true, Ordering::Relaxed);

                    if !last_listener_run || listeners.is_some() {
                        root.apply_notion(Rc::new(Deferred::<T>::Outdated {
                            input: input.clone_rc(),
                        }) as Rc<dyn Any>);
                    }
                })))
            }

            let output = T::run(&states, input.clone_rc()).await;

            if !listener_run.load(Ordering::Relaxed) {
                let _result = listeners.borrow_mut().replace(states.take_listeners());
            }

            root.apply_notion(Rc::new(Deferred::<T>::Completed { input, output }) as Rc<dyn Any>);
        });
    })
}

/// A hook to connect to an `InputSelector`.
///
/// An input selector is similar to a selector, but also with an input.
///
/// Its value will be automatically re-calculated when any state used in the selector has changed.
///
/// Returns a [`Rc<T>`].
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// #
/// # enum SliceAction {
/// #     Increment,
/// # }
/// #
/// #[derive(Default, PartialEq, Slice)]
/// struct Value(i64);
/// #
/// # impl Reducible for Value {
/// #     type Action = SliceAction;
/// #
/// #     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
/// #         match action {
/// #             Self::Action::Increment => Self(self.0 + 1).into(),
/// #         }
/// #     }
/// # }
///
/// #[derive(PartialEq)]
/// pub struct DivBy {
///     inner: bool,
/// }
///
/// impl InputSelector for DivBy {
///     type Input = i64;
///
///     fn select(states: &BounceStates, input: Rc<Self::Input>) -> Rc<Self> {
///         let val = states.get_slice_value::<Value>();
///
///         Self {
///             inner: val.0 % *input == 0,
///         }
///         .into()
///     }
/// }
/// # #[function_component(ShowIsEven)]
/// # fn show_is_even() -> Html {
/// let is_even = use_input_selector_value::<DivBy>(2.into());
/// # Html::default()
/// # }
/// ```
pub fn use_input_selector_value<T>(input: Rc<T::Input>) -> Rc<T>
where
    T: InputSelector + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    let val = {
        let input = input.clone();
        let root = root.clone();
        use_state_eq(move || {
            let states = root.states();

            root.get_state::<InputSelectorsState<T>>()
                .get_state(input)
                .get(states)
        })
    };

    {
        let val = val.clone();
        let root = root;
        use_effect_with_deps(
            move |(root, input)| {
                let listener = root
                    .get_state::<InputSelectorsState<T>>()
                    .get_state(input.clone())
                    .listen(Rc::new(Callback::from(move |m| {
                        val.set(m);
                    })));

                move || {
                    std::mem::drop(listener);
                }
            },
            (root, input),
        );
    }
    (*val).clone()
}

/// A hook to connect to a `Selector`.
///
/// A selector is a derived state which its value is derived from other states.
///
/// Its value will be automatically re-calculated when any state used in the selector has changed.
///
/// Returns a [`Rc<T>`].
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// #
/// # enum SliceAction {
/// #     Increment,
/// # }
/// #
/// #[derive(Default, PartialEq, Slice)]
/// struct Value(i64);
/// #
/// # impl Reducible for Value {
/// #     type Action = SliceAction;
/// #
/// #     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
/// #         match action {
/// #             Self::Action::Increment => Self(self.0 + 1).into(),
/// #         }
/// #     }
/// # }
///
/// #[derive(PartialEq)]
/// pub struct IsEven {
///     inner: bool,
/// }
///
/// impl Selector for IsEven {
///     fn select(states: &BounceStates) -> Rc<Self> {
///         let val = states.get_slice_value::<Value>();
///
///         Self {
///             inner: val.0 % 2 == 0,
///         }
///         .into()
///     }
/// }
/// # #[function_component(ShowIsEven)]
/// # fn show_is_even() -> Html {
/// let is_even = use_selector_value::<IsEven>();
/// # Html::default()
/// # }
/// ```
pub fn use_selector_value<T>() -> Rc<T>
where
    T: Selector + 'static,
{
    use_input_selector_value::<UnitSelector<T>>(().into())
        .inner
        .clone()
}
