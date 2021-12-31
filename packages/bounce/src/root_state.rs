use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use anymap2::any::CloneAny;
use anymap2::Map;

use crate::any_state::AnyState;
use crate::atom::Atom;
use crate::slice::{Slice, SliceState};
use crate::utils::Id;

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
        }
    }
}

impl PartialEq for BounceRootState {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

#[derive(Clone, PartialEq)]
pub struct BounceStates {
    inner: BounceRootState,
}

impl BounceStates {
    pub fn get_slice<T>(&self) -> Rc<T>
    where
        T: Slice + 'static,
    {
        self.inner.get_state::<SliceState<T>>().get()
    }

    pub fn get_atom<T>(&self) -> Rc<T>
    where
        T: Atom + 'static,
    {
        self.get_slice::<T>()
    }
}

impl fmt::Debug for BounceStates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BounceStates")
            .field("inner", &"BounceRootState")
            .finish()
    }
}
