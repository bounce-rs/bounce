use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::provider::BounceRootState;
use crate::state::Stateful;

pub fn use_bounce_value<T>() -> Rc<T>
where
    T: Stateful + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    let val = {
        let root = root.clone();
        use_state(move || -> Rc<T> { root.get_state::<T>() })
    };

    {
        let val = val.clone();
        use_state(move || {
            root.listen::<T, _>(move |root| {
                val.set(root.get_state::<T>());
            })
        });
    }

    (*val).clone()
}

pub fn use_set_bounce_value<T>() -> Rc<dyn Fn(T::Input)>
where
    T: Stateful + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    let state = use_state(move || {
        Rc::new(move |input: T::Input| {
            root.set_state::<T>(input);
        })
    });

    (*state).clone()
}

pub struct UseBounceStateHandle<T>
where
    T: Stateful,
{
    inner: Rc<T>,
    root: BounceRootState,
}

impl<T> UseBounceStateHandle<T>
where
    T: Stateful + 'static,
{
    pub fn set(&self, input: T::Input) {
        self.root.set_state::<T>(input);
    }
}

impl<T> Deref for UseBounceStateHandle<T>
where
    T: Stateful,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Clone for UseBounceStateHandle<T>
where
    T: Stateful,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            root: self.root.clone(),
        }
    }
}

impl<T> fmt::Debug for UseBounceStateHandle<T>
where
    T: Stateful + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseBounceStateHandle")
            .field("inner", &self.inner)
            .finish()
    }
}

pub fn use_bounce_state<T>() -> UseBounceStateHandle<T>
where
    T: Stateful + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    let val = {
        let root = root.clone();
        use_state(move || -> Rc<T> { root.get_state::<T>() })
    };

    {
        let val = val.clone();
        let root = root.clone();
        use_state(move || {
            root.listen::<T, _>(move |root| {
                val.set(root.get_state::<T>());
            })
        });
    }

    let val = (*val).clone();

    UseBounceStateHandle { inner: val, root }
}
