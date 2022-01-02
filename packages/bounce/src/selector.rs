use std::rc::Rc;

use crate::input_selector::InputSelector;
use crate::root_state::BounceStates;

/// An auto-updating derived state.
///
/// It will automatically update when any selected state changes and only notifies registered
/// hooks when `prev_value != next_value`.
pub trait Selector: PartialEq {
    /// Selects `self` from existing bounce states.
    ///
    /// # Panics
    ///
    /// `states.get_selector_value::<T>()` will panic if you are trying to create a loop by selecting current selector
    /// again.
    fn select(states: &BounceStates) -> Rc<Self>;
}

#[derive(PartialEq)]
pub(crate) struct UnitSelector<T>
where
    T: Selector + 'static,
{
    pub inner: Rc<T>,
}

impl<T> InputSelector for UnitSelector<T>
where
    T: Selector + 'static,
{
    type Input = ();

    fn select(states: &BounceStates, _input: Rc<()>) -> Rc<Self> {
        Self {
            inner: T::select(states),
        }
        .into()
    }
}
