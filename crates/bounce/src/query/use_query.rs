use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use yew::platform::pinned::oneshot;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionHandle, SuspensionResult};

use super::query_states::{
    QuerySelector, QuerySlice, QuerySliceAction, QuerySliceValue, RunQuery, RunQueryInput,
};
use super::traits::{Query, QueryResult};
use crate::states::future_notion::use_future_notion_runner;
use crate::states::input_selector::use_input_selector_value;
use crate::states::slice::use_slice_dispatch;
use crate::utils::Id;

pub(super) enum QueryMemoValue<T: Query + 'static> {
    Suspended {
        suspension: Suspension,
        _handle: Option<SuspensionHandle>,
    },
    Ready {
        id: Id,
        state: Rc<QueryState<T>>,
    },
}

/// Query State
#[derive(Debug, PartialEq)]
pub enum QueryState<T>
where
    T: Query + 'static,
{
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

impl<T> Clone for QueryState<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Completed { result } => Self::Completed {
                result: result.clone(),
            },
            Self::Refreshing { last_result } => Self::Refreshing {
                last_result: last_result.clone(),
            },
        }
    }
}

impl<T> PartialEq<&QueryState<T>> for QueryState<T>
where
    T: Query + 'static,
{
    fn eq(&self, other: &&QueryState<T>) -> bool {
        self == *other
    }
}

impl<T> PartialEq<QueryState<T>> for &'_ QueryState<T>
where
    T: Query + 'static,
{
    fn eq(&self, other: &QueryState<T>) -> bool {
        *self == other
    }
}

/// A handle returned by [`use_query`].
pub struct UseQueryHandle<T>
where
    T: Query + 'static,
{
    pub(super) input: Rc<T::Input>,
    pub(super) state_id: Id,
    pub(super) state: Rc<QueryState<T>>,
    pub(super) run_query: Rc<dyn Fn(RunQueryInput<T>)>,
    pub(super) dispatch_state: Rc<dyn Fn(QuerySliceAction<T>)>,
}

impl<T> UseQueryHandle<T>
where
    T: Query + 'static,
{
    /// Returns the state of current query.
    pub fn state(&self) -> &QueryState<T> {
        self.state.as_ref()
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

impl<T> Clone for UseQueryHandle<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            state: self.state.clone(),
            state_id: self.state_id,
            run_query: self.run_query.clone(),
            dispatch_state: self.dispatch_state.clone(),
        }
    }
}

impl<T> Deref for UseQueryHandle<T>
where
    T: Query + 'static,
{
    type Target = QueryResult<T>;

    fn deref(&self) -> &Self::Target {
        match self.state() {
            QueryState::Completed { result } => result,
            QueryState::Refreshing { last_result } => last_result,
        }
    }
}

impl<T> fmt::Debug for UseQueryHandle<T>
where
    T: Query + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseQueryHandle")
            .field("value", self.deref())
            .finish()
    }
}

/// A hook to run a query and subscribes to its result, suspending while fetching.
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
/// use bounce::query::{Query, QueryResult, use_query};
/// use yew::prelude::*;
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
/// fn comp() -> HtmlResult {
///     let user = use_query::<UserQuery>(0.into())?;
///
///     match user.as_ref() {
///         // The result is Some(Ok(_)) if the query has loaded successfully.
///         Ok(m) => Ok(html! {<div>{"User's name is "}{m.value.name.to_string()}</div>}),
///         // The result is Some(Err(_)) if an error is returned during fetching.
///         Err(_e) => Ok(html! {<div>{"Oops, something went wrong."}</div>}),
///     }
/// }
/// ```
#[hook]
pub fn use_query<T>(input: Rc<T::Input>) -> SuspensionResult<UseQueryHandle<T>>
where
    T: Query + 'static,
{
    let id = *use_memo((), |_| Id::new());
    let value_state = use_input_selector_value::<QuerySelector<T>>(input.clone());
    let dispatch_state = use_slice_dispatch::<QuerySlice<T>>();
    let run_query = use_future_notion_runner::<RunQuery<T>>();

    // Produce a Suspension or a ready value. When the value is not yet available,
    // the query is initiated as part of constructing the Suspension (following
    // the same pattern as Yew's use_future_with).
    let value = use_memo((input.clone(), value_state.clone()), {
        let run_query = run_query.clone();
        move |(input, value_state): &(Rc<T::Input>, Rc<QuerySelector<T>>)| match value_state.value {
            None => {
                let (sender, receiver) = oneshot::channel();
                run_query(RunQueryInput {
                    id,
                    input: input.clone(),
                    sender: Rc::new(RefCell::new(Some(sender))),
                    is_refresh: false,
                });
                QueryMemoValue::Suspended {
                    suspension: Suspension::from_future(async move {
                        let _ = receiver.await;
                    }),
                    _handle: None,
                }
            }
            Some(QuerySliceValue::Loading { .. }) => {
                let (suspension, handle) = Suspension::new();
                QueryMemoValue::Suspended {
                    suspension,
                    _handle: Some(handle),
                }
            }
            Some(QuerySliceValue::Completed { id, ref result }) => QueryMemoValue::Ready {
                id,
                state: Rc::new(QueryState::Completed {
                    result: result.clone(),
                }),
            },
            Some(QuerySliceValue::Outdated { id, ref result }) => QueryMemoValue::Ready {
                id,
                state: Rc::new(QueryState::Refreshing {
                    last_result: result.clone(),
                }),
            },
        }
    });

    {
        let input = input.clone();
        let run_query = run_query.clone();

        use_effect_with((id, input, value_state), move |(id, input, value_state)| {
            if matches!(value_state.value, Some(QuerySliceValue::Outdated { .. })) {
                run_query(RunQueryInput {
                    id: *id,
                    input: input.clone(),
                    sender: Rc::default(),
                    is_refresh: false,
                });
            }

            || {}
        });
    }

    match value.as_ref() {
        QueryMemoValue::Ready {
            id: state_id,
            state,
        } => Ok(UseQueryHandle {
            state: state.clone(),
            state_id: *state_id,
            input,
            dispatch_state,
            run_query,
        }),
        QueryMemoValue::Suspended { suspension, .. } => Err(suspension.clone()),
    }
}
