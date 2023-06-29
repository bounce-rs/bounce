use std::any::{Any, TypeId};
use std::rc::Rc;

use anymap2::AnyMap;

/// A common trait for all states.
pub(crate) trait AnyState {
    /// Applies a notion.
    fn apply(&self, notion: Rc<dyn Any>);

    /// Returns a list of notion ids that this state accepts.
    fn notion_ids(&self) -> Vec<TypeId> {
        Vec::new()
    }

    /// Creates a state from a possible initialise value.
    fn create(init_states: &mut AnyMap) -> Self
    where
        Self: Sized;
}
