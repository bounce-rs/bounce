use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use anymap2::any::CloneAny;
use anymap2::Map;
use yew::callback::Callback;

use crate::any_state::AnyState;
use crate::atom::Atom;
use crate::selector::{Selector, SelectorState};
use crate::slice::{Slice, SliceState};
use crate::utils::Id;
use crate::utils::Listener;

pub(crate) type StateMap = Map<dyn CloneAny>;

#[derive(Clone)]
pub(crate) struct BounceRootState {
    id: Id,
    states: Rc<RefCell<StateMap>>,
    any_states: Rc<RefCell<Vec<Rc<dyn AnyState>>>>,
}

impl Default for BounceRootState {
    fn default() -> Self {
        Self {
            id: Id::new(),
            states: Rc::default(),
            any_states: Rc::default(),
        }
    }
}

impl BounceRootState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_state<T>(&self) -> T
    where
        T: AnyState + Clone + Default + 'static,
    {
        let mut states = self.states.borrow_mut();
        if let Some(m) = states.get::<T>().cloned() {
            m
        } else {
            let state = T::default();
            states.insert(state.clone());

            let mut any_states = self.any_states.borrow_mut();
            any_states.push(Rc::new(state.clone()) as Rc<dyn AnyState>);

            state
        }
    }

    pub fn get_any_states(&self) -> Vec<Rc<dyn AnyState>> {
        self.any_states.borrow().clone()
    }

    pub fn apply_notion(&self, notion: Rc<dyn Any>) {
        let any_states = self.get_any_states();

        for any_state in any_states.iter() {
            any_state.apply(notion.clone());
        }
    }

    pub fn states(&self) -> BounceStates {
        BounceStates {
            inner: self.clone(),
            listeners: Rc::default(),
            listener_callbacks: Rc::default(),
        }
    }
}

impl PartialEq for BounceRootState {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

/// A type to access states under a bounce root.
pub struct BounceStates {
    inner: BounceRootState,
    listeners: Rc<RefCell<Vec<Listener>>>,
    listener_callbacks: Rc<RefCell<Vec<Rc<Callback<()>>>>>,
}

impl BounceStates {
    /// Returns the value of a [`Slice`].
    pub fn get_slice_value<T>(&self) -> Rc<T>
    where
        T: Slice + 'static,
    {
        let state = self.inner.get_state::<SliceState<T>>();
        let listener_callbacks = self.listener_callbacks.borrow().clone();
        let mut listeners = Vec::new();

        for callback in listener_callbacks {
            let listener = state.listen(Rc::new(Callback::from(move |_: Rc<T>| {
                callback.emit(());
            })));

            listeners.push(listener);
        }

        self.listeners.borrow_mut().extend(listeners);

        state.get()
    }

    /// Returns the value of an [`Atom`].
    pub fn get_atom_value<T>(&self) -> Rc<T>
    where
        T: Atom + 'static,
    {
        self.get_slice_value::<T>()
    }

    /// Returns the value of a [`Selector`].
    pub fn get_selector_value<T>(&self) -> Rc<T>
    where
        T: Selector + 'static,
    {
        let state = self.inner.get_state::<SelectorState<T>>();
        let listener_callbacks = self.listener_callbacks.borrow().clone();
        let mut listeners = Vec::new();

        for callback in listener_callbacks {
            let listener = state.listen(Rc::new(Callback::from(move |_: Rc<T>| {
                callback.emit(());
            })));

            listeners.push(listener);
        }

        self.listeners.borrow_mut().extend(listeners);

        state.get(self.derived_clone())
    }

    pub(crate) fn add_listener_callback(&self, callback: Rc<Callback<()>>) {
        let mut listener_callbacks = self.listener_callbacks.borrow_mut();
        listener_callbacks.push(callback);
    }

    pub(crate) fn take_listeners(&self) -> Vec<Listener> {
        let mut next_listeners = Vec::new();
        let mut last_listeners = self.listeners.borrow_mut();

        std::mem::swap(&mut next_listeners, &mut last_listeners);

        next_listeners
    }

    /// Creates a sub-states, but with a separate listener holder.
    fn derived_clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            listeners: Rc::default(),
            listener_callbacks: Rc::default(),
        }
    }
}

impl fmt::Debug for BounceStates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BounceStates")
            .field("inner", &"BounceRootState")
            .finish()
    }
}
