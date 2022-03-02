/// A trait to be notified when the state value changes.
///
/// Currently, only Slices and Atoms can be observed. This API may be expanded to other state types
/// in the future.
pub trait Observed {
    /// Notified when the state is initialised.
    fn registered(&self) {}
    /// Notified when the state value has changed.
    fn changed(&self) {}
}
