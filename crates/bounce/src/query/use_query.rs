use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use yew::platform::pinned::oneshot;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

use super::query_states::{
    QuerySelector, QueryState, QueryStateAction, QueryStateValue, RunQuery, RunQueryInput,
};
use super::traits::{Query, QueryResult};
use crate::states::future_notion::use_future_notion_runner;
use crate::states::input_selector::use_input_selector_value;
use crate::states::slice::use_slice_dispatch;
use crate::utils::Id;

/// A handle returned by [`use_query`].
pub struct UseQueryHandle<T>
where
    T: Query + 'static,
{
    pub(super) input: Rc<T::Input>,
    pub(super) state_id: Id,
    pub(super) result: QueryResult<T>,
    pub(super) run_query: Rc<dyn Fn(RunQueryInput<T>)>,
    pub(super) dispatch_state: Rc<dyn Fn(QueryStateAction<T>)>,
}

impl<T> UseQueryHandle<T>
where
    T: Query + 'static,
{
    /// Refreshes the query.
    ///
    /// The query will be refreshed with the input provided to the hook.
    pub async fn refresh(&self) -> QueryResult<T> {
        (self.dispatch_state)(QueryStateAction::Refresh(
            (self.state_id, self.input.clone()).into(),
        ));

        let id = Id::new();

        let (sender, receiver) = oneshot::channel();

        (self.run_query)(RunQueryInput {
            id,
            input: self.input.clone(),
            sender: Rc::new(RefCell::new(Some(sender))),
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
            result: self.result.clone(),
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
        &self.result
    }
}

impl<T> fmt::Debug for UseQueryHandle<T>
where
    T: Query + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseQueryHandle")
            .field("value", &self.result)
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
    let id = *use_memo(|_| Id::new(), ());
    let value_state = use_input_selector_value::<QuerySelector<T>>(input.clone());
    let dispatch_state = use_slice_dispatch::<QueryState<T>>();
    let run_query = use_future_notion_runner::<RunQuery<T>>();

    let value = use_memo(
        |v| match v.value {
            Some(QueryStateValue::Loading(_)) | None => Err(Suspension::new()),
            Some(QueryStateValue::Completed { id, result: ref m })
            | Some(QueryStateValue::Outdated((id, ref m))) => Ok((id, m.clone())),
        },
        value_state.clone(),
    );

    {
        let input = input.clone();
        let run_query = run_query.clone();

        use_memo(
            move |_| {
                run_query(RunQueryInput {
                    id,
                    input: input.clone(),
                    sender: Rc::default(),
                });
            },
            (),
        );
    }

    {
        let input = input.clone();
        let run_query = run_query.clone();

        use_effect_with_deps(
            move |(id, input, value_state)| {
                if matches!(value_state.value, Some(QueryStateValue::Outdated(_))) {
                    run_query(RunQueryInput {
                        id: *id,
                        input: input.clone(),
                        sender: Rc::default(),
                    });
                }

                || {}
            },
            (id, input, value_state),
        );
    }

    value
        .as_ref()
        .as_ref()
        .map(|(id, value)| UseQueryHandle {
            state_id: *id,
            input,
            dispatch_state,
            run_query,
            result: value.clone(),
        })
        .map_err(|(s, _)| s.clone())
}
