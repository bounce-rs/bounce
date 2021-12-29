use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use anymap2::any::CloneAny;
use anymap2::Map;
use yew::prelude::*;

use crate::any_state::AnyState;
use crate::slice::{Slice, SliceListener, SliceState};
use crate::utils::Id;

pub(crate) type StateMap = Map<dyn CloneAny>;

/// Properties for [`BounceRoot`].
#[derive(Properties, Debug, PartialEq)]
pub struct BounceRootProps {
    #[prop_or_default]
    pub children: Children,
}

#[derive(Clone)]
pub(crate) struct BounceRootState {
    id: Id,
    states: Rc<RefCell<StateMap>>,
    any_states: Rc<RefCell<Vec<Box<dyn AnyState>>>>,
}

impl BounceRootState {
    pub fn dispatch_action<T>(&self, val: T::Action)
    where
        T: Slice + 'static,
    {
        let mut states = self.states.borrow_mut();
        if let Some(m) = states.get::<SliceState<T>>().cloned() {
            m.dispatch(val)
        } else {
            let state = SliceState::<T>::default();
            states.insert(state.clone());
            state.dispatch(val);

            let mut any_states = self.any_states.borrow_mut();
            any_states.push(Box::new(state) as Box<dyn AnyState>);
        }
    }

    pub fn listen<T, CB>(&self, callback: CB) -> SliceListener<T>
    where
        T: Slice + 'static,
        CB: Fn(Rc<T>) + 'static,
    {
        let cb = Rc::new(Callback::from(callback));

        let mut states = self.states.borrow_mut();
        let state = states.remove::<SliceState<T>>().unwrap_or_default();
        let listener = state.listen(cb);

        states.insert(state);

        listener
    }

    pub fn get<T>(&self) -> Rc<T>
    where
        T: Slice + 'static,
    {
        let mut states = self.states.borrow_mut();
        if let Some(m) = states.get::<SliceState<T>>().cloned() {
            m.get()
        } else {
            let state = SliceState::<T>::default();
            states.insert(state.clone());

            let mut any_states = self.any_states.borrow_mut();
            any_states.push(Box::new(state.clone()) as Box<dyn AnyState>);

            state.get()
        }
    }

    pub fn apply_notion(&self, notion: Rc<dyn Any>) {
        let any_states = self.any_states.borrow();

        for any_state in any_states.iter() {
            any_state.apply(notion.clone());
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
        states: Rc::default(),
        any_states: Rc::default(),
    });

    html! {
        <ContextProvider<BounceRootState> context={(*root_state).clone()}>{children}</ContextProvider<BounceRootState>>
    }
}
