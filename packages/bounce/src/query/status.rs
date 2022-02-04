/// Query Status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum QueryStatus {
    /// The query is idling.
    ///
    /// This status is currently only used by mutations that has yet to be started.
    Idle, // paused for queries, not started for mutations
    /// The query is loading.
    Loading,
    /// The query is successful.
    Ok,
    /// The query has failed with an Error.
    Err,
}
