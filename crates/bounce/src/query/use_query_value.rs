use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use yew::platform::pinned::oneshot;
use yew::prelude::*;

use super::query_states::{
    QuerySelector, QuerySlice, QuerySliceAction, QuerySliceValue, RunQuery, RunQueryInput,
};

use super::traits::{Query, QueryResult};
use crate::states::future_notion::use_future_notion_runner;
use crate::states::input_selector::use_input_selector_value;
use crate::states::slice::use_slice_dispatch;
use crate::utils::Id;

/// Query Value State
#[derive(Debug, PartialEq)]
pub enum QueryValueState<T>
where
    T: Query + 'static,
{
    /// The query is loading.
    Loading,
    /// The query has completed.
    Completed {
        /// Result of the completed query.
        result: QueryResult<T>,
    },
    /// A previous query has completed and a new query is currently loading.
    Refreshing {
        /// Result of last completed query.
        last_result: QueryResult<T>,
    },
}

impl<T> Clone for QueryValueState<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Loading => Self::Loading,
            Self::Completed { result } => Self::Completed {
                result: result.clone(),
            },
            Self::Refreshing { last_result } => Self::Refreshing {
                last_result: last_result.clone(),
            },
        }
    }
}

/// A handle returned by [`use_query_value`].
pub struct UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    input: Rc<T::Input>,
    state: Rc<QueryValueState<T>>,
    run_query: Rc<dyn Fn(RunQueryInput<T>)>,
    dispatch_state: Rc<dyn Fn(QuerySliceAction<T>)>,
}

impl<T> UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    /// Returns the state of current query.
    pub fn state(&self) -> &QueryValueState<T> {
        self.state.as_ref()
    }

    /// Returns the result of current query (if any).
    ///
    /// - `None` indicates that the query is currently loading.
    /// - `Some(Ok(m))` indicates that the query is successful and the content is stored in `m`.
    /// - `Some(Err(e))` indicates that the query has failed and the error is stored in `e`.
    pub fn result(&self) -> Option<&QueryResult<T>> {
        match self.state() {
            QueryValueState::Completed { result, .. }
            | QueryValueState::Refreshing {
                last_result: result,
                ..
            } => Some(result),
            _ => None,
        }
    }

    /// Refreshes the query.
    ///
    /// The query will be refreshed with the input provided to the hook.
    pub async fn refresh(&self) -> QueryResult<T> {
        let id = Id::new();
        (self.dispatch_state)(QuerySliceAction::Refresh {
            id,
            input: self.input.clone(),
        });

        let (sender, receiver) = oneshot::channel();

        (self.run_query)(RunQueryInput {
            id,
            input: self.input.clone(),
            sender: Rc::new(RefCell::new(Some(sender))),
            is_refresh: true,
        });

        receiver.await.unwrap()
    }
}

impl<T> Clone for UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            state: self.state.clone(),
            run_query: self.run_query.clone(),
            dispatch_state: self.dispatch_state.clone(),
        }
    }
}

impl<T> fmt::Debug for UseQueryValueHandle<T>
where
    T: Query + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseQueryValueHandle")
            .field("value", &self.state)
            .finish()
    }
}

/// A hook to run a query and subscribes to its result.
///
/// A query is a state that is cached by an Input and queried automatically upon initialisation of the
/// state and re-queried when the input changes.
///
/// Queries are usually tied to idempotent methods like `GET`, which means that they should be side-effect
/// free and can be cached.
///
/// If your endpoint modifies data, then you need to use a [mutation](super::use_mutation_value).
///
/// # Example
///
/// ```
/// use std::rc::Rc;
/// use std::convert::Infallible;
/// use bounce::prelude::*;
/// use bounce::query::{Query, QueryResult, use_query_value};
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
///
/// #[function_component(Comp)]
/// fn comp() -> Html {
///     let user = use_query_value::<UserQuery>(0.into());
///
///     match user.result() {
///         // The result is None if the query is currently loading.
///         None => html! {<div>{"loading..."}</div>},
///         // The result is Some(Ok(_)) if the query has loaded successfully.
///         Some(Ok(m)) => html! {<div>{"User's name is "}{m.value.name.to_string()}</div>},
///         // The result is Some(Err(_)) if an error is returned during fetching.
///         Some(Err(e)) => html! {<div>{"Oops, something went wrong."}</div>},
///     }
/// }
/// ```
#[hook]
pub fn use_query_value<T>(input: Rc<T::Input>) -> UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    let id = *use_memo(|_| Id::new(), ());
    let value = use_input_selector_value::<QuerySelector<T>>(input.clone());
    let dispatch_state = use_slice_dispatch::<QuerySlice<T>>();
    let run_query = use_future_notion_runner::<RunQuery<T>>();

    {
        let input = input.clone();
        let run_query = run_query.clone();
        use_effect_with_deps(
            move |(id, input, value)| {
                if value.is_none() || matches!(value, Some(QuerySliceValue::Outdated { .. })) {
                    run_query(RunQueryInput {
                        id: *id,
                        input: input.clone(),
                        sender: Rc::default(),
                        is_refresh: false,
                    });
                }

                || {}
            },
            (id, input, value.value.clone()),
        );
    }

    let state = use_memo(
        |value| match value.value {
            Some(QuerySliceValue::Completed { ref result, .. }) => QueryValueState::Completed {
                result: result.clone(),
            },
            Some(QuerySliceValue::Outdated { ref result, .. }) => QueryValueState::Refreshing {
                last_result: result.clone(),
            },
            Some(QuerySliceValue::Loading { .. }) | None => QueryValueState::Loading,
        },
        value,
    );

    UseQueryValueHandle {
        input,
        dispatch_state,
        run_query,
        state,
    }
}
