use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use anymap2::AnyMap;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::any_state::AnyState;
use crate::root_state::BounceRootState;
use crate::utils::{notify_listeners, Listener, ListenerVec};

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

    /// Returns a list of notion ids that this Slice accepts.
    fn notion_ids(&self) -> Vec<TypeId>;

    /// Notifies a slice that it has changed.
    fn changed(self: Rc<Self>) {}

    /// Creates a new Slice with its initial value.
    fn create(init_states: &mut AnyMap) -> Self
    where
        Self: 'static + Sized,
    {
        init_states.remove().unwrap_or_default()
    }
}

/// A trait to provide cloning on slices.
///
/// This trait provides a `self.clone_slice()` method that can be used as an alias of `(*self).clone()`
/// in reduce and apply functions to produce a owned clone of the slice.
pub trait CloneSlice: Slice + Clone {
    /// Clones current slice.
    #[inline]
    fn clone_slice(&self) -> Self {
        self.clone()
    }
}

impl<T> CloneSlice for T where T: Slice + Clone {}

#[derive(Debug, Default)]
pub(crate) struct SliceState<T>
where
    T: Slice,
{
    value: Rc<RefCell<Rc<T>>>,
    listeners: Rc<RefCell<ListenerVec<T>>>,
}

impl<T> Clone for SliceState<T>
where
    T: Slice,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            listeners: self.listeners.clone(),
        }
    }
}

impl<T> SliceState<T>
where
    T: Slice + 'static,
{
    pub fn dispatch(&self, action: T::Action) {
        let maybe_next_val = {
            let mut value = self.value.borrow_mut();
            let prev_val: Rc<T> = value.clone();
            let next_val = prev_val.clone().reduce(action);

            let should_notify = prev_val != next_val;
            *value = next_val.clone();

            should_notify.then(|| next_val)
        };

        if let Some(next_val) = maybe_next_val {
            self.notify_listeners(next_val);
        }
    }

    pub fn notify_listeners(&self, val: Rc<T>) {
        val.clone().changed();
        notify_listeners(self.listeners.clone(), val);
    }

    pub fn listen(&self, callback: Rc<Callback<Rc<T>>>) -> Listener {
        let mut callbacks_ref = self.listeners.borrow_mut();
        callbacks_ref.push(Rc::downgrade(&callback));

        Listener::new(callback)
    }

    pub fn get(&self) -> Rc<T> {
        let value = self.value.borrow();
        value.clone()
    }
}

impl<T> AnyState for SliceState<T>
where
    T: Slice + 'static,
{
    fn apply(&self, notion: Rc<dyn Any>) {
        let maybe_next_val = {
            let mut value = self.value.borrow_mut();
            let prev_val: Rc<T> = value.clone();
            let next_val = prev_val.clone().apply(notion);

            let should_notify = prev_val != next_val;
            *value = next_val.clone();

            should_notify.then(|| next_val)
        };

        if let Some(next_val) = maybe_next_val {
            self.notify_listeners(next_val);
        }
    }

    fn notion_ids(&self) -> Vec<TypeId> {
        self.value.borrow().notion_ids()
    }

    fn create(init_states: &mut AnyMap) -> Self
    where
        Self: Sized,
    {
        Self {
            value: Rc::new(RefCell::new(T::create(init_states).into())),
            listeners: Rc::default(),
        }
    }
}

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

/// A hook to connect to a [`Slice`](macro@crate::Slice).
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
///     };
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
#[hook]
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
        use_memo(
            move |root| {
                let state = root.get_state::<SliceState<T>>();

                // we need to set the value here again in case the value has changed between the
                // initial render and the listener is registered.
                val.set(state.get());

                state.listen(Rc::new(Callback::from(move |m| {
                    val.set(m);
                })))
            },
            root,
        );
    }

    let val = (*val).clone();

    UseSliceHandle { inner: val, root }
}

/// A hook to produce a dispatch function for a [`Slice`](macro@crate::Slice).
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
#[hook]
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

/// A read-only hook to connect to the value of a [`Slice`](macro@crate::Slice).
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
#[hook]
pub fn use_slice_value<T>() -> Rc<T>
where
    T: Slice + 'static,
{
    use_slice::<T>().inner
}
