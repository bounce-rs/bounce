use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use anymap2::any::CloneAny;
use anymap2::{Entry, Map};
use yew::callback::Callback;

use crate::any_state::AnyState;
use crate::states::atom::{Atom, AtomSlice};
use crate::states::input_selector::{InputSelector, InputSelectorsState};
use crate::states::selector::{Selector, UnitSelector};
use crate::states::slice::{Slice, SliceState};
use crate::utils::Id;
use crate::utils::Listener;

pub(crate) type StateMap = Map<dyn CloneAny>;
type AnyStateMap = HashMap<TypeId, Vec<Rc<dyn AnyState>>>;

#[derive(Clone)]
pub(crate) struct BounceRootState {
    id: Id,
    states: Rc<RefCell<StateMap>>,
    notion_states: Rc<RefCell<AnyStateMap>>,
}

impl Default for BounceRootState {
    fn default() -> Self {
        Self {
            id: Id::new(),
            states: Rc::default(),
            notion_states: Rc::default(),
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

        match states.entry::<T>() {
            Entry::Occupied(m) => m.get().clone(),
            Entry::Vacant(m) => {
                let state = T::default();
                m.insert(state.clone());

                let mut notion_states = self.notion_states.borrow_mut();
                for notion_id in state.notion_ids() {
                    match notion_states.entry(notion_id) {
                        hash_map::Entry::Occupied(mut m) => {
                            m.get_mut().push(Rc::new(state.clone()) as Rc<dyn AnyState>);
                        }
                        hash_map::Entry::Vacant(m) => {
                            m.insert(vec![Rc::new(state.clone()) as Rc<dyn AnyState>]);
                        }
                    }
                }

                state
            }
        }
    }

    pub fn apply_notion<T>(&self, notion: Rc<T>)
    where
        T: 'static,
    {
        let notion_states = self.notion_states.borrow();

        let notion = notion as Rc<dyn Any>;

        if let Some(m) = notion_states.get(&TypeId::of::<T>()) {
            for any_state in m.iter() {
                any_state.apply(notion.clone());
            }
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
    /// Returns the value of a `Slice`.
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

    /// Returns the value of an `Atom`.
    pub fn get_atom_value<T>(&self) -> Rc<T>
    where
        T: Atom + 'static,
    {
        self.get_slice_value::<AtomSlice<T>>().inner.clone()
    }

    /// Returns the value of an [`InputSelector`].
    pub fn get_input_selector_value<T>(&self, input: Rc<T::Input>) -> Rc<T>
    where
        T: InputSelector + 'static,
    {
        let state = self
            .inner
            .get_state::<InputSelectorsState<T>>()
            .get_state(input);
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

    /// Returns the value of a [`Selector`].
    pub fn get_selector_value<T>(&self) -> Rc<T>
    where
        T: Selector + 'static,
    {
        self.get_input_selector_value::<UnitSelector<T>>(Rc::new(()))
            .inner
            .clone()
    }

    pub(crate) fn add_listener_callback(&self, callback: Rc<Callback<()>>) {
        let mut listener_callbacks = self.listener_callbacks.borrow_mut();
        listener_callbacks.push(callback);
    }

    pub(crate) fn take_listeners(&self) -> Vec<Listener> {
        let mut next_listeners = Vec::new();
        let mut last_listeners = self.listeners.borrow_mut();

        std::mem::swap(&mut next_listeners, &mut last_listeners);

        // Also clears callbacks.
        let mut listener_callbacks = self.listener_callbacks.borrow_mut();
        listener_callbacks.clear();

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
