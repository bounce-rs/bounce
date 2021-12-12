use std::any::TypeId;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use anymap2::any::CloneAny;
use anymap2::Map;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::slice::Slice;
use crate::utils::Id;

pub(crate) type SliceMap = Map<dyn CloneAny>;
type ListenerVec = Vec<Weak<Callback<BounceRootState>>>;
type ListenerMap = Rc<RefCell<HashMap<TypeId, ListenerVec>>>;

/// Properties for [`BounceRoot`].
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
    slices: Rc<RefCell<SliceMap>>,
    listeners: ListenerMap,
}

impl BounceRootState {
    pub(crate) fn dispatch_action<T>(&self, val: T::Action)
    where
        T: Slice + 'static,
    {
        let should_notify = {
            let mut atoms = self.slices.borrow_mut();
            let prev_val = atoms.remove::<Rc<T>>().unwrap_or_default();
            let next_val = prev_val.clone().reduce(val);

            let should_notify = prev_val != next_val;

            atoms.insert(next_val);

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

    pub(crate) fn get<T>(&self) -> Rc<T>
    where
        T: Slice + 'static,
    {
        let mut atoms = self.slices.borrow_mut();
        if let Some(m) = atoms.get::<Rc<T>>().cloned() {
            m
        } else {
            let val = Rc::new(T::default());
            atoms.insert(val.clone());
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

/// A `<BounceRoot />`.
///
/// For bounce states to function, A `<BounceRoot />` must present and registered as a context
/// provider.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # use bounce::BounceRoot;
/// #[function_component(App)]
/// fn app() -> Html {
///     html! {
///         <BounceRoot>
///             // children...
///         </BounceRoot>
///     }
/// }
///
/// ```
#[function_component(BounceRoot)]
pub fn bounce_root(props: &BounceRootProps) -> Html {
    let children = props.children.clone();

    let root_state = use_state(|| BounceRootState {
        id: Id::new(),
        slices: Rc::default(),
        listeners: Rc::default(),
    });

    html! {
        <ContextProvider<BounceRootState> context={(*root_state).clone()}>{children}</ContextProvider<BounceRootState>>
    }
}
