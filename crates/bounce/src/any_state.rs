use std::any::Any;
use std::rc::Rc;

/// A common trait for all states.
pub(crate) trait AnyState {
    /// Applies a notion.
    fn apply(&self, notion: Rc<dyn Any>);
}
