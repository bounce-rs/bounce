use std::rc::Rc;

/// A trait to apply a notion on a state.
///
/// See: [`use_notion_applier`](crate::use_notion_applier)
pub trait WithNotion<T: 'static> {
    /// Applies a notion on current state.
    ///
    /// This always yields a new instance of [`Rc<Self>`] so it can be compared with the previous
    /// state using [`PartialEq`].
    fn apply(self: Rc<Self>, notion: Rc<T>) -> Rc<Self>;
}
