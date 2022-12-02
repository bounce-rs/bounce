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

mod mutation;
mod prepared_query;
mod query_;
mod query_value;
mod status;
mod traits;

pub use mutation::{use_mutation_value, UseMutationValueHandle};
pub use prepared_query::use_prepared_query;
pub use query_::{use_query, UseQueryHandle};
pub use query_value::{use_query_value, UseQueryValueHandle};
pub use status::QueryStatus;
pub use traits::{Mutation, MutationResult, Query, QueryResult};
