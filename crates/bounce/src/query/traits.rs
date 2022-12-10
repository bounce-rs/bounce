use async_trait::async_trait;
use std::hash::Hash;
use std::rc::Rc;

use crate::root_state::BounceStates;

/// A Result returned by queries.
pub type QueryResult<T> = std::result::Result<Rc<T>, <T as Query>::Error>;

/// A trait to be implemented on queries.
///
/// # Note
///
/// This trait is implemented with [async_trait](macro@async_trait), you should apply an `#[async_trait(?Send)]`
/// attribute to your implementation of this trait.
///
/// # Example
///
/// ```
/// use std::rc::Rc;
/// use std::convert::Infallible;
/// use bounce::prelude::*;
/// use bounce::query::{Query, QueryResult};
/// use yew::prelude::*;
/// use async_trait::async_trait;
///
/// #[derive(Debug, PartialEq)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// #[derive(Debug, PartialEq)]
/// struct UserQuery {
///     value: User
/// }
///
/// #[async_trait(?Send)]
/// impl Query for UserQuery {
///     type Input = u64;
///     type Error = Infallible;
///
///     async fn query(_states: &BounceStates, input: Rc<u64>) -> QueryResult<Self> {
///         // fetch user
///
///         Ok(UserQuery{ value: User { id: *input, name: "John Smith".into() } }.into())
///     }
/// }
/// ```
///
/// See: [`use_query`](super::use_query()) and [`use_query_value`](super::use_query_value())
#[async_trait(?Send)]
pub trait Query: PartialEq {
    /// The Input type of a query.
    ///
    /// The input type must implement Hash and Eq as it is used as the key of results in a
    /// HashMap.
    type Input: Hash + Eq + 'static;

    /// The Error type of a query.
    type Error: 'static + std::error::Error + PartialEq + Clone;

    /// Runs a query.
    ///
    /// This method will only be called when the result is not already cached.
    ///
    /// # Note
    ///
    /// When implementing this method with async_trait, you can use the following function
    /// signature:
    ///
    /// ```ignore
    /// async fn query(states: &BounceStates, input: Rc<Self::Input>) -> QueryResult<Self>
    /// ```
    async fn query(states: &BounceStates, input: Rc<Self::Input>) -> QueryResult<Self>;
}

/// A Result returned by mutations.
pub type MutationResult<T> = std::result::Result<Rc<T>, <T as Mutation>::Error>;

/// A trait to be implemented on mutations.
///
/// # Note
///
/// This trait is implemented with [async_trait](macro@async_trait), you should apply an `#[async_trait(?Send)]`
/// attribute to your implementation of this trait.
///
/// # Example
///
/// ```
/// use std::rc::Rc;
/// use std::convert::Infallible;
/// use bounce::prelude::*;
/// use bounce::query::{Mutation, MutationResult};
/// use yew::prelude::*;
/// use async_trait::async_trait;
///
/// #[derive(Debug, PartialEq)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// #[derive(Debug, PartialEq)]
/// struct UpdateUserMutation {
/// }
///
/// #[async_trait(?Send)]
/// impl Mutation for UpdateUserMutation {
///     type Input = User;
///     type Error = Infallible;
///
///     async fn run(_states: &BounceStates, _input: Rc<User>) -> MutationResult<Self> {
///         // updates the user information.
///
///         Ok(UpdateUserMutation {}.into())
///     }
/// }
/// ```
///
/// See: [`use_mutation`](super::use_mutation())
#[async_trait(?Send)]
pub trait Mutation: PartialEq {
    /// The Input type.
    type Input: 'static;

    /// The Error type.
    type Error: 'static + std::error::Error + PartialEq + Clone;

    /// Runs a mutation.
    ///
    /// # Note
    ///
    /// When implementing this method with async_trait, you can use the following function
    /// signature:
    ///
    /// ```ignore
    /// async fn run(states: &BounceStates, input: Rc<Self::Input>) -> MutationResult<Self>
    /// ```
    async fn run(states: &BounceStates, input: Rc<Self::Input>) -> MutationResult<Self>;
}
