use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

use async_trait::async_trait;
use futures::channel::oneshot;
use yew::prelude::*;

use super::status::QueryStatus;
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
/// See: [`use_query_value`]
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

struct RunQueryInput<T>
where
    T: Query + 'static,
{
    id: Id,
    input: Rc<T::Input>,
    sender: RunQuerySender<T>,
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
struct IsCurrentQuery<T>
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
            let current_id = match m {
                QueryStateValue::Loading(id) => id,
                QueryStateValue::Completed((id, _)) => id,
            };

            return Self {
                _marker: PhantomData,
                inner: *current_id == id,
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
async fn run_query<T>(states: &BounceStates, input: &RunQueryInput<T>) -> Option<QueryResult<T>>
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
}

impl<T> QueryStateValue<T>
where
    T: Query + 'static,
{
    fn id(&self) -> Id {
        match self {
            Self::Loading(ref id) => *id,
            Self::Completed(ref m) => m.0,
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
        }
    }
}

enum QueryStateAction<T>
where
    T: Query + 'static,
{
    Refresh(Rc<(Id, Rc<T::Input>)>),
}

#[derive(Slice)]
#[with_notion(Deferred<RunQuery<T>>)]
struct QueryState<T>
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
                if self.queries.contains_key(&input) {
                    return self;
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
                if let Some(QueryStateValue::Completed((ref m, _))) = self.queries.get(&input) {
                    if m == &id {
                        return self;
                    }
                }

                let mut queries = self.queries.clone();
                queries.remove(&input);

                Self {
                    ctr: self.ctr + 1,
                    queries,
                }
                .into()
            }
        }
    }
}

#[derive(PartialEq)]
struct QuerySelector<T>
where
    T: Query + 'static,
{
    value: Option<QueryStateValue<T>>,
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

/// A handle returned by [`use_query_value`].
pub struct UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    input: Rc<T::Input>,
    value: Option<QueryStateValue<T>>,
    run_query: Rc<dyn Fn(RunQueryInput<T>)>,
    dispatch_state: Rc<dyn Fn(QueryStateAction<T>)>,
}

impl<T> UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    /// Returns the status of current query.
    pub fn status(&self) -> QueryStatus {
        match self.value {
            Some(QueryStateValue::Completed((_, Ok(_)))) => QueryStatus::Ok,
            Some(QueryStateValue::Completed((_, Err(_)))) => QueryStatus::Err,
            Some(QueryStateValue::Loading(_)) => QueryStatus::Loading,
            None => QueryStatus::Idle,
        }
    }

    /// Returns the result of current query (if any).
    ///
    /// - `None` indicates that the query is currently loading.
    /// - `Some(Ok(m))` indicates that the query is successful and the content is stored in `m`.
    /// - `Some(Err(e))` indicates that the query has failed and the error is stored in `e`.
    pub fn result(&self) -> Option<QueryResult<T>> {
        match self.value {
            Some(QueryStateValue::Completed((_, ref m))) => Some(m.clone()),
            _ => None,
        }
    }

    /// Refreshes the query.
    ///
    /// The query will be refreshed with the input provided to the hook.
    pub async fn refresh(&self) -> QueryResult<T> {
        if let Some(ref m) = self.value {
            (self.dispatch_state)(QueryStateAction::Refresh(
                (m.id(), self.input.clone()).into(),
            ));
        }

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

impl<T> Clone for UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            value: self.value.clone(),
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
            .field("value", &self.value)
            .finish()
    }
}

/// A hook to run a query and subscribes to its result.
///
/// A query is a state that cached by an Input and queried automatically upon initialisation of the
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
pub fn use_query_value<T>(input: Rc<T::Input>) -> UseQueryValueHandle<T>
where
    T: Query + 'static,
{
    let id = *use_ref(Id::new);
    let value = use_input_selector_value::<QuerySelector<T>>(input.clone());
    let dispatch_state = use_slice_dispatch::<QueryState<T>>();
    let run_query = use_future_notion_runner::<RunQuery<T>>();

    {
        let input = input.clone();
        let run_query = run_query.clone();
        use_effect_with_deps(
            move |(id, input)| {
                run_query(RunQueryInput {
                    id: *id,
                    input: input.clone(),
                    sender: Rc::default(),
                });

                || {}
            },
            (id, input),
        );
    }

    UseQueryValueHandle {
        input,
        dispatch_state,
        run_query,
        value: value.value.clone(),
    }
}
