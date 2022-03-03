use std::rc::Rc;

/// A trait to be notified when the state value changes.
///
/// Currently, only Slices and Atoms can be observed. This API may be expanded to other state types
/// in the future.
///
/// # Example
///
/// ```
/// use bounce::prelude::*;
///
/// #[derive(Atom, PartialEq, Default)]
/// #[observed] // observed states need to be denoted with the observed attribute.
/// struct State {
///     value: usize,
/// }
///
/// impl Observed for State {
///     fn changed(self: Rc<Self>) {
///         // this method will be called when the value of the state changes.
///     }
/// }
/// ```
pub trait Observed {
    /// Notified when the state value has changed.
    fn changed(self: Rc<Self>);
}
