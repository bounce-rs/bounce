/// Query Status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum QueryStatus {
    // Implementation Note: paused for queries, not started for mutations
    // query pausing is yet to be implemented.
    /// The query is idling.
    ///
    /// This status is currently only used by mutations that has yet to be started.
    ///
    Idle,
    /// The query is loading.
    Loading,
    /// The query is refreshing.
    Refreshing,
    /// The query is successful.
    Ok,
    /// The query has failed with an Error.
    Err,
}
