use std::rc::Rc;

use crate::input_selector::{use_input_selector_value, InputSelector};
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

/// A hook to connect to a [`Selector`].
///
/// A selector is a derived state which its value is derived from other states.
///
/// Its value will be automatically re-calculated when any state used in the selector has changed.
///
/// Returns a [`Rc<T>`].
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// #
/// # enum SliceAction {
/// #     Increment,
/// # }
/// #
/// #[derive(Default, PartialEq, Slice)]
/// struct Value(i64);
/// #
/// # impl Reducible for Value {
/// #     type Action = SliceAction;
/// #
/// #     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
/// #         match action {
/// #             Self::Action::Increment => Self(self.0 + 1).into(),
/// #         }
/// #     }
/// # }
///
/// #[derive(PartialEq)]
/// pub struct IsEven {
///     inner: bool,
/// }
///
/// impl Selector for IsEven {
///     fn select(states: &BounceStates) -> Rc<Self> {
///         let val = states.get_slice_value::<Value>();
///
///         Self {
///             inner: val.0 % 2 == 0,
///         }
///         .into()
///     }
/// }
/// # #[function_component(ShowIsEven)]
/// # fn show_is_even() -> Html {
/// let is_even = use_selector_value::<IsEven>();
/// # Html::default()
/// # }
/// ```
pub fn use_selector_value<T>() -> Rc<T>
where
    T: Selector + 'static,
{
    use_input_selector_value::<UnitSelector<T>>(().into())
        .inner
        .clone()
}
