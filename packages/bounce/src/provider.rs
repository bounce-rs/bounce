use std::any::TypeId;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use anymap2::any::CloneAny;
use anymap2::Map;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::state::{State, Stateful};
use crate::utils::Id;

pub(crate) type StateMap = Map<dyn CloneAny>;
type ListenerVec = Vec<Weak<Callback<BounceRootState>>>;
type ListenerMap = Rc<RefCell<HashMap<TypeId, ListenerVec>>>;

#[derive(Properties, Debug, PartialEq)]
pub struct BounceRootProps {
    #[prop_or_default]
    pub children: Children,
}

pub struct SliceListener {
    _listener: Rc<Callback<BounceRootState>>,
}

#[derive(Clone)]
pub(crate) struct BounceRootState {
    id: Id,
    states: Rc<RefCell<StateMap>>,
    listeners: ListenerMap,
}

impl BounceRootState {
    pub(crate) fn set_state<T>(&self, val: T::Input)
    where
        T: Stateful + 'static,
    {
        let should_notify = {
            let mut state = {
                let mut states = self.states.borrow_mut();
                states
                    .remove::<T::State>()
                    .unwrap_or_else(|| T::State::new(self.clone().into()))
            };

            let should_notify = state.set(val);

            let mut states = self.states.borrow_mut();
            states.insert(state);
            should_notify
        };

        if should_notify {
            self.notify_listeners::<T>();
        }
    }

    pub(crate) fn listen<T, CB>(&self, callback: CB) -> SliceListener
    where
        T: 'static,
        CB: Fn(BounceRootState) + 'static,
    {
        let cb = Rc::new(Callback::from(callback));

        let type_id = TypeId::of::<T>();

        let mut listeners = self.listeners.borrow_mut();

        if let Entry::Vacant(e) = listeners.entry(type_id) {
            e.insert(vec![Rc::downgrade(&cb)]);
        } else {
            let listeners = listeners.get_mut(&type_id).unwrap_throw();
            listeners.push(Rc::downgrade(&cb));
        };

        SliceListener { _listener: cb }
    }

    pub(crate) fn get_state<T>(&self) -> Rc<T>
    where
        T: Stateful + 'static,
    {
        if let Some(mut m) = {
            let states = self.states.borrow_mut();
            states.get::<T::State>().cloned()
        } {
            m.get()
        } else {
            let mut state = T::State::new(self.clone().into());
            let val = state.get();

            let mut states = self.states.borrow_mut();
            states.insert(state);
            val
        }
    }

    pub(crate) fn notify_listeners<T>(&self)
    where
        T: 'static,
    {
        let callables = {
            let mut callbacks_ref = self.listeners.borrow_mut();

            let callbacks_ref = match callbacks_ref.get_mut(&TypeId::of::<T>()) {
                Some(m) => m,
                None => return,
            };

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
            callback.emit(self.to_owned())
        }
    }
}

impl PartialEq for BounceRootState {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

#[function_component(BounceRoot)]
pub fn bounce_root(props: &BounceRootProps) -> Html {
    let children = props.children.clone();

    let root_state = use_state(|| BounceRootState {
        id: Id::new(),
        states: Rc::default(),
        listeners: Rc::default(),
    });

    html! {
        <ContextProvider<BounceRootState> context={(*root_state).clone()}>{children}</ContextProvider<BounceRootState>>
    }
}
