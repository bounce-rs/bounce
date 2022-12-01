use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use async_trait::async_trait;
use yew::platform::pinned::oneshot;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

use crate::future_notion;
use crate::root_state::BounceStates;
use crate::states::future_notion::{use_future_notion_runner, Deferred};
use crate::states::input_selector::{use_input_selector_value, InputSelector};
use crate::states::notion::WithNotion;
use crate::states::slice::{use_slice_dispatch, Slice};
use crate::utils::Id;

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
/// See: [`use_query`] and [`use_query_value`](super::use_query_value)
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

type RunQuerySender<T> = Rc<RefCell<Option<oneshot::Sender<QueryResult<T>>>>>;

pub(super) struct RunQueryInput<T>
where
    T: Query + 'static,
{
    pub id: Id,
    pub input: Rc<T::Input>,
    pub sender: RunQuerySender<T>,
}

impl<T> Clone for RunQueryInput<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            input: self.input.clone(),
            sender: self.sender.clone(),
        }
    }
}

#[derive(PartialEq)]
pub(super) struct IsCurrentQuery<T>
where
    T: Query + 'static,
{
    _marker: PhantomData<T>,
    inner: bool,
}

impl<T> InputSelector for IsCurrentQuery<T>
where
    T: Query + 'static,
{
    type Input = (Id, Rc<T::Input>);

    fn select(states: &BounceStates, input: Rc<(Id, Rc<T::Input>)>) -> Rc<Self> {
        let (id, input) = (*input).clone();

        if let Some(m) = states
            .get_slice_value::<QueryState<T>>()
            .queries
            .get(&input)
        {
            let current_id = m.id();

            return Self {
                _marker: PhantomData,
                inner: current_id == id,
            }
            .into();
        }

        Self {
            _marker: PhantomData,
            inner: false,
        }
        .into()
    }
}

#[future_notion(RunQuery)]
pub(super) async fn run_query<T>(
    states: &BounceStates,
    input: &RunQueryInput<T>,
) -> Option<QueryResult<T>>
where
    T: Query + 'static,
{
    let is_current_query = states
        .get_input_selector_value::<IsCurrentQuery<T>>((input.id, input.input.clone()).into());

    if !is_current_query.inner {
        return None;
    }

    let result = T::query(states, input.input.clone()).await;

    if let Some(m) = input.sender.borrow_mut().take() {
        let _result = m.send(result.clone());
    }

    Some(result)
}

#[derive(PartialEq, Debug)]
pub enum QueryStateValue<T>
where
    T: Query + 'static,
{
    Loading(Id),
    Completed((Id, QueryResult<T>)),
    Outdated((Id, QueryResult<T>)),
}

impl<T> QueryStateValue<T>
where
    T: Query + 'static,
{
    pub(crate) fn id(&self) -> Id {
        match self {
            Self::Loading(ref id) => *id,
            Self::Completed(ref m) => m.0,
            Self::Outdated(ref m) => m.0,
        }
    }
}

impl<T> Clone for QueryStateValue<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Loading(ref id) => Self::Loading(*id),
            Self::Completed(ref m) => Self::Completed(m.clone()),
            Self::Outdated(ref m) => Self::Outdated(m.clone()),
        }
    }
}

pub(super) enum QueryStateAction<T>
where
    T: Query + 'static,
{
    Refresh(Rc<(Id, Rc<T::Input>)>),
}

#[derive(Slice)]
#[bounce(with_notion(Deferred<RunQuery<T>>))]
pub(super) struct QueryState<T>
where
    T: Query + 'static,
{
    ctr: u64,
    queries: HashMap<Rc<T::Input>, QueryStateValue<T>>,
}

impl<T> Reducible for QueryState<T>
where
    T: Query + 'static,
{
    type Action = QueryStateAction<T>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Refresh(input) => {
                let (id, input) = (*input).clone();

                if self.queries.get(&input).map(|m| m.id()) == Some(id) {
                    let mut queries = self.queries.clone();

                    queries.remove(&input);

                    return Self {
                        ctr: self.ctr + 1,
                        queries,
                    }
                    .into();
                }

                self
            }
        }
    }
}

impl<T> Default for QueryState<T>
where
    T: Query + 'static,
{
    fn default() -> Self {
        Self {
            ctr: 0,
            queries: HashMap::new(),
        }
    }
}

impl<T> PartialEq for QueryState<T>
where
    T: Query + 'static,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.ctr == rhs.ctr
    }
}

impl<T> WithNotion<Deferred<RunQuery<T>>> for QueryState<T>
where
    T: Query + 'static,
{
    fn apply(self: Rc<Self>, notion: Rc<Deferred<RunQuery<T>>>) -> Rc<Self> {
        match *notion {
            Deferred::Pending { ref input } => {
                let RunQueryInput { input, id, .. } = (**input).clone();
                if let Some(m) = self.queries.get(&input) {
                    if !matches!(m, QueryStateValue::Outdated(_)) {
                        return self;
                    }
                }

                let mut queries = self.queries.clone();
                queries.insert(input, QueryStateValue::Loading(id));

                Self {
                    ctr: self.ctr + 1,
                    queries,
                }
                .into()
            }
            Deferred::Completed {
                ref input,
                ref output,
            } => {
                let RunQueryInput { input, id, .. } = (**input).clone();
                if let Some(ref output) = **output {
                    let mut queries = self.queries.clone();
                    queries.insert(input, QueryStateValue::Completed((id, (*output).clone())));

                    Self {
                        ctr: self.ctr + 1,
                        queries,
                    }
                    .into()
                } else {
                    self
                }
            }
            Deferred::Outdated { ref input } => {
                let RunQueryInput { input, id, .. } = (**input).clone();
                if let Some(QueryStateValue::Completed(ref val)) = self.queries.get(&input) {
                    if val.0 == id {
                        let mut queries = self.queries.clone();
                        queries.insert(input.clone(), QueryStateValue::Outdated(val.clone()));

                        return Self {
                            ctr: self.ctr + 1,
                            queries,
                        }
                        .into();
                    }
                }

                self
            }
        }
    }
}

#[derive(PartialEq)]
pub(super) struct QuerySelector<T>
where
    T: Query + 'static,
{
    pub value: Option<QueryStateValue<T>>,
}

impl<T> InputSelector for QuerySelector<T>
where
    T: Query + 'static,
{
    type Input = T::Input;

    fn select(states: &BounceStates, input: Rc<T::Input>) -> Rc<Self> {
        let value = states
            .get_slice_value::<QueryState<T>>()
            .queries
            .get(&input)
            .cloned();

        Self { value }.into()
    }
}

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
            Some(QueryStateValue::Completed((id, ref m)))
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
