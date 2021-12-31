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
    any_states: Rc<RefCell<Vec<Rc<dyn AnyState>>>>,
}

impl BounceRootState {
    fn get_state<T>(&self) -> T
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

    fn get_any_states(&self) -> Vec<Rc<dyn AnyState>> {
        self.any_states.borrow().clone()
    }

    pub fn dispatch_action<T>(&self, val: T::Action)
    where
        T: Slice + 'static,
    {
        let state = self.get_state::<SliceState<T>>();
        state.dispatch(val);
    }

    pub fn listen<T, CB>(&self, callback: CB) -> SliceListener<T>
    where
        T: Slice + 'static,
        CB: Fn(Rc<T>) + 'static,
    {
        let cb = Rc::new(Callback::from(callback));

        let state = self.get_state::<SliceState<T>>();

        state.listen(cb)
    }

    pub fn get<T>(&self) -> Rc<T>
    where
        T: Slice + 'static,
    {
        let state = self.get_state::<SliceState<T>>();
        state.get()
    }

    pub fn apply_notion(&self, notion: Rc<dyn Any>) {
        let any_states = self.get_any_states();

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
