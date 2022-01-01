use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use yew::callback::Callback;

use crate::any_state::AnyState;
use crate::root_state::BounceStates;
use crate::utils::{Listener, ListenerVec};

/// An auto-updating derived state.
///
/// It will automatically update when any selected state changes and only notifies registered
/// hooks when `prev_value != next_value`.
pub trait Selector: PartialEq {
    /// Selects `self` from existing bounce states.
    ///
    /// # Panics
    ///
    /// `states.get_selector_value::<T>()` will panic if you are trying to create a loop by selecting current selector
    /// again.
    fn select(states: &BounceStates) -> Rc<Self>;
}

#[derive(Debug)]
pub(crate) struct SelectorState<T>
where
    T: Selector,
{
    value: Rc<RefCell<Option<Rc<T>>>>,
    listeners: Rc<RefCell<ListenerVec<T>>>,
    state_listener_handles: Rc<RefCell<Vec<Listener>>>,
    states: Rc<RefCell<Option<Rc<BounceStates>>>>, // reference cycle?
}

impl<T> Default for SelectorState<T>
where
    T: Selector,
{
    fn default() -> Self {
        Self {
            value: Rc::default(),
            listeners: Rc::default(),
            state_listener_handles: Rc::default(),
            states: Rc::default(),
        }
    }
}

impl<T> Clone for SelectorState<T>
where
    T: Selector,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            listeners: self.listeners.clone(),
            state_listener_handles: self.state_listener_handles.clone(),
            states: self.states.clone(),
        }
    }
}

impl<T> SelectorState<T>
where
    T: Selector + 'static,
{
    pub fn select_value(&self, states: &BounceStates) -> Rc<T> {
        let self_ = self.clone();
        states.add_listener_callback(Rc::new(Callback::from(move |_: ()| {
            self_.clone().refresh();
        })));

        let next_value = T::select(states);

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
        let callables = {
            let mut callbacks_ref = self.listeners.borrow_mut();

            // Any gone weak references are removed when called.
            let (callbacks, callbacks_weak) = callbacks_ref.iter().cloned().fold(
                (Vec::new(), Vec::new()),
                |(mut callbacks, mut callbacks_weak), m| {
                    if let Some(m_strong) = m.clone().upgrade() {
                        callbacks.push(m_strong);
                        callbacks_weak.push(m);
                    }

                    (callbacks, callbacks_weak)
                },
            );

            *callbacks_ref = callbacks_weak;

            callbacks
        };

        for callback in callables {
            callback.emit(val.clone())
        }
    }

    pub fn listen(&self, callback: Rc<Callback<Rc<T>>>) -> Listener {
        let mut callbacks_ref = self.listeners.borrow_mut();
        callbacks_ref.push(Rc::downgrade(&callback));

        Listener::new(callback)
    }
}

impl<T> AnyState for SelectorState<T>
where
    T: Selector + 'static,
{
    fn apply(&self, _notion: Rc<dyn Any>) {}
}
