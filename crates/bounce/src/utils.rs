use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use yew::callback::Callback;

#[derive(PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
pub struct Id(u64);

impl Default for Id {
    fn default() -> Self {
        static CTR: Lazy<AtomicU64> = Lazy::new(AtomicU64::default);

        Self(CTR.fetch_add(1, Ordering::SeqCst))
    }
}

impl Id {
    pub fn new() -> Self {
        Self::default()
    }
}

pub(crate) struct Listener {
    _listener: Rc<dyn Any>,
}

impl Listener {
    pub fn new(inner: Rc<dyn Any>) -> Self {
        Self { _listener: inner }
    }
}

impl fmt::Debug for Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Listener").finish()
    }
}

pub(crate) type ListenerVec<T> = Vec<Weak<Callback<Rc<T>>>>;

pub(crate) fn notify_listeners<T>(listeners: Rc<RefCell<ListenerVec<T>>>, val: Rc<T>) {
    let callables = {
        let mut callbacks_ref = listeners.borrow_mut();

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
