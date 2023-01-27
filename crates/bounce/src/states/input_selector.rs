use std::any::Any;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use anymap2::AnyMap;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::any_state::AnyState;
use crate::root_state::{BounceRootState, BounceStates};
use crate::utils::{notify_listeners, Listener, ListenerVec};

/// An auto-updating derived state, similar to [`Selector`](crate::Selector), but with an input.
///
/// Each selector with a different input are treated as a different selector.
///
/// It will automatically update when any selected state changes and only notifies registered
/// hooks when `prev_value != next_value`.
pub trait InputSelector: PartialEq {
    /// The input type of the current input selector.
    type Input: 'static + Eq + Hash;

    /// Selects `self` from existing bounce states with an input.
    ///
    /// # Panics
    ///
    /// `states.get_selector_value::<T>()` will panic if you are trying to create a loop by selecting current selector
    /// again.
    fn select(states: &BounceStates, input: Rc<Self::Input>) -> Rc<Self>;
}

#[derive(Debug)]
pub(crate) struct InputSelectorState<T>
where
    T: InputSelector,
{
    input: Rc<T::Input>,
    value: Rc<RefCell<Option<Rc<T>>>>,
    listeners: Rc<RefCell<ListenerVec<T>>>,
    state_listener_handles: Rc<RefCell<Vec<Listener>>>,
    states: Rc<RefCell<Option<Rc<BounceStates>>>>,
}

impl<T> Clone for InputSelectorState<T>
where
    T: InputSelector,
{
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            value: self.value.clone(),
            listeners: self.listeners.clone(),
            state_listener_handles: self.state_listener_handles.clone(),
            states: self.states.clone(),
        }
    }
}

impl<T> InputSelectorState<T>
where
    T: InputSelector + 'static,
{
    pub fn new(input: Rc<T::Input>) -> Self {
        Self {
            input,
            value: Rc::default(),
            listeners: Rc::default(),
            state_listener_handles: Rc::default(),
            states: Rc::default(),
        }
    }

    pub fn select_value(&self, states: &BounceStates) -> Rc<T> {
        let self_ = self.clone();
        states.add_listener_callback(Rc::new(Callback::from(move |_: ()| {
            self_.clone().refresh();
        })));

        let next_value = T::select(states, self.input.clone());

        let mut handles = self.state_listener_handles.borrow_mut();
        *handles = states.take_listeners();

        next_value
    }

    pub fn get(&self, states: BounceStates) -> Rc<T> {
        let mut value = self.value.borrow_mut();

        match value.clone() {
            Some(m) => m,
            None => {
                let next_value = self.select_value(&states);
                let mut last_states = self.states.borrow_mut();
                *last_states = Some(Rc::new(states));
                *value = Some(next_value.clone());

                next_value
            }
        }
    }

    pub fn refresh(&self) {
        if let Some(states) = self.states.borrow().clone() {
            let maybe_next_val = {
                let mut value = self.value.borrow_mut();
                let prev_val = value.clone();
                let next_val = self.select_value(&states);

                let should_notify = prev_val.as_ref() != Some(&next_val);
                *value = Some(next_val.clone());

                should_notify.then(|| next_val)
            };

            if let Some(next_val) = maybe_next_val {
                self.notify_listeners(next_val);
            }
        }
    }

    pub fn notify_listeners(&self, val: Rc<T>) {
        notify_listeners(self.listeners.clone(), val);
    }

    pub fn listen(&self, callback: Rc<Callback<Rc<T>>>) -> Listener {
        let mut callbacks_ref = self.listeners.borrow_mut();
        callbacks_ref.push(Rc::downgrade(&callback));

        Listener::new(callback)
    }
}

pub(crate) type InputSelectorMap<T> =
    HashMap<Rc<<T as InputSelector>::Input>, InputSelectorState<T>>;

pub(crate) struct InputSelectorsState<T>
where
    T: InputSelector + 'static,
{
    selectors: Rc<RefCell<InputSelectorMap<T>>>,
}

impl<T> InputSelectorsState<T>
where
    T: InputSelector + 'static,
{
    pub fn get_state(&self, input: Rc<T::Input>) -> InputSelectorState<T> {
        let mut selectors = self.selectors.borrow_mut();

        match selectors.entry(input.clone()) {
            Entry::Occupied(m) => m.get().clone(),
            Entry::Vacant(m) => {
                let state = InputSelectorState::<T>::new(input);
                m.insert(state.clone());
                state
            }
        }
    }
}

impl<T> Default for InputSelectorsState<T>
where
    T: InputSelector,
{
    fn default() -> Self {
        Self {
            selectors: Rc::default(),
        }
    }
}

impl<T> Clone for InputSelectorsState<T>
where
    T: InputSelector,
{
    fn clone(&self) -> Self {
        Self {
            selectors: self.selectors.clone(),
        }
    }
}

impl<T> AnyState for InputSelectorsState<T>
where
    T: InputSelector + 'static,
{
    fn apply(&self, _notion: Rc<dyn Any>) {}

    fn create(_init_states: &mut AnyMap) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

/// A hook to connect to an [`InputSelector`].
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
#[hook]
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
        use_memo(
            move |(root, input)| {
                let state = root
                    .get_state::<InputSelectorsState<T>>()
                    .get_state(input.clone());

                // we need to set the value here again in case the value has changed between the
                // initial render and the listener is registered.
                val.set(state.get(root.states()));

                state.listen(Rc::new(Callback::from(move |m| {
                    val.set(m);
                })))
            },
            (root, input),
        );
    }
    (*val).clone()
}
