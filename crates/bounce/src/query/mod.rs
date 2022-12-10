//! A module to provide helper states to facilitate data fetching.
//!
//! It provides hook-based access to APIs with automatic caching and request deduplication backed
//! by Bounce's state management mechanism.
//!
//! This module is inspired by [RTK Query](https://redux-toolkit.js.org/rtk-query/overview).
//!
//! There are two methods to interact with APIs: [Query](use_query_value) and
//! [Mutation](use_mutation_value)
//!
//! # Note
//!
//! Bounce does not provide an implementation of HTTP Client.
//!
//! You can use reqwest or gloo-net if you need a generic HTTP Client.
//!
//! If your backend is GraphQL, you can use graphql-client in conjunction with reqwest.

mod mutation_states;
mod query_states;
mod status;
mod traits;
mod use_mutation;
mod use_prepared_query;
mod use_query;
mod use_query_value;

pub use status::QueryStatus;
pub use traits::{Mutation, MutationResult, Query, QueryResult};
pub use use_mutation::{use_mutation, UseMutationHandle};
pub use use_prepared_query::use_prepared_query;
pub use use_query::{use_query, UseQueryHandle};
pub use use_query_value::{use_query_value, UseQueryValueHandle};
