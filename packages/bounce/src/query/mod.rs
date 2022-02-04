//! A module to provide helper states to facilitate data fetching.
//!
//! There are two methods to interact with APIs: [Query](use_query_value) and
//! [Mutation](use_mutation_value)
//!
//! # Note
//!
//! Bounce does not provide an implementation of HTTP Client.
//!
//! You can use reqwest or reqwasm if you need a generic HTTP Client.
//!
//! If your backend is GraphQL, you can use graphql-client in conjunction with reqwest.

mod mutation;
mod query_;
mod status;

pub use mutation::{use_mutation_value, Mutation, MutationResult, UseMutationValueHandle};
pub use query_::{use_query_value, Query, QueryResult, UseQueryValueHandle};
pub use status::QueryStatus;
