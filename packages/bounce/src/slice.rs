use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use yew::callback::Callback;

use crate::any_state::AnyState;

pub(crate) type ListenerVec<T> = Vec<Weak<Callback<Rc<T>>>>;

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
}

/// A trait to provide cloning on slices.
///
/// This trait provides a `self.clone_slice()` method that can be used as an alias of `(*self).clone()`
/// in reduce functions to produce a owned clone of the slice.
pub trait CloneSlice: Slice + Clone {
    /// Clones current slice.
    #[inline]
    fn clone_slice(&self) -> Self {
        self.clone()
    }
}

impl<T> CloneSlice for T where T: Slice + Clone {}

pub(crate) struct SliceListener<T> {
    _listener: Rc<Callback<Rc<T>>>,
}

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
    T: Slice,
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

    pub fn listen(&self, callback: Rc<Callback<Rc<T>>>) -> SliceListener<T> {
        let mut callbacks_ref = self.listeners.borrow_mut();
        callbacks_ref.push(Rc::downgrade(&callback));

        SliceListener {
            _listener: callback,
        }
    }

    pub fn get(&self) -> Rc<T> {
        let value = self.value.borrow();
        value.clone()
    }
}

impl<T> AnyState for SliceState<T>
where
    T: Slice,
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
}
