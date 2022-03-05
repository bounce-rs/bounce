use std::any::{Any, TypeId};
use std::rc::Rc;

/// A common trait for all states.
pub(crate) trait AnyState {
    /// Applies a notion.
    fn apply(&self, notion: Rc<dyn Any>);

    /// Returns a list of notion ids that this state accepts.
    fn notion_ids(&self) -> Vec<TypeId> {
        Vec::new()
    }
}
