use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use yew::platform::pinned::oneshot;
use yew::prelude::*;

use super::traits::{Query, QueryResult};
use crate::future_notion;
use crate::root_state::BounceStates;
use crate::states::future_notion::Deferred;
use crate::states::input_selector::InputSelector;
use crate::states::notion::WithNotion;
use crate::states::slice::Slice;
use crate::utils::Id;

type RunQuerySender<T> = Rc<RefCell<Option<oneshot::Sender<QueryResult<T>>>>>;

pub(super) struct RunQueryInput<T>
where
    T: Query + 'static,
{
    pub id: Id,
    pub input: Rc<T::Input>,
    pub sender: RunQuerySender<T>,
    pub is_refresh: bool,
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
            is_refresh: self.is_refresh,
        }
    }
}

pub(super) struct IsCurrentQuery<T>
where
    T: Query + 'static,
{
    _marker: PhantomData<T>,
    inner: bool,
}

impl<T> PartialEq for IsCurrentQuery<T>
where
    T: Query + 'static,
{
    fn eq(&self, _other: &Self) -> bool {
        // We do not want to subscribe to this state, so we always return true for partial equality.
        true
    }
}

impl<T> InputSelector for IsCurrentQuery<T>
where
    T: Query + 'static,
{
    type Input = (Id, Rc<T::Input>);

    fn select(states: &BounceStates, input: Rc<(Id, Rc<T::Input>)>) -> Rc<Self> {
        let (id, input) = input.as_ref().clone();

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

#[future_notion]
pub(super) async fn RunQuery<T>(
    states: &BounceStates,
    input: &RunQueryInput<T>,
) -> Option<QueryResult<T>>
where
    T: Query + 'static,
{
    let RunQueryInput {
        id,
        input,
        sender,
        is_refresh,
    } = input.clone();

    let is_current_query =
        states.get_input_selector_value::<IsCurrentQuery<T>>((id, input.clone()).into());

    if !is_current_query.inner && !is_refresh {
        // We drop the channel.
        sender.borrow_mut().take();

        return None;
    }

    let result = T::query(states, input.clone()).await;

    if let Some(m) = sender.borrow_mut().take() {
        let _result = m.send(result.clone());
    }

    Some(result)
}

#[derive(PartialEq, Debug)]
pub enum QueryStateValue<T>
where
    T: Query + 'static,
{
    Loading { id: Id },
    Completed { id: Id, result: QueryResult<T> },
    Outdated { id: Id, result: QueryResult<T> },
}

impl<T> QueryStateValue<T>
where
    T: Query + 'static,
{
    pub(crate) fn id(&self) -> Id {
        match self {
            Self::Loading { ref id }
            | Self::Completed { ref id, .. }
            | Self::Outdated { ref id, .. } => *id,
        }
    }
}

impl<T> Clone for QueryStateValue<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Loading { id } => Self::Loading { id: *id },
            Self::Completed { id, ref result } => Self::Completed {
                id: *id,
                result: result.clone(),
            },
            Self::Outdated { id, ref result } => Self::Outdated {
                id: *id,
                result: result.clone(),
            },
        }
    }
}

pub(super) enum QueryStateAction<T>
where
    T: Query + 'static,
{
    Refresh {
        id: Id,
        input: Rc<T::Input>,
    },
    LoadPrepared {
        id: Id,
        input: Rc<T::Input>,
        result: QueryResult<T>,
    },
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

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Refresh { input, id } => {
                let this = Rc::make_mut(&mut self);
                this.ctr += 1;

                // Make the query as outdated.
                if let Some(m) = this.queries.get_mut(&input) {
                    if let QueryStateValue::Completed { result, .. } = m.clone() {
                        *m = QueryStateValue::Outdated { id, result }
                    }
                }
            }

            Self::Action::LoadPrepared { id, input, result } => {
                if self.queries.get(&input).is_none() {
                    let this = Rc::make_mut(&mut self);
                    this.ctr += 1;

                    if let Entry::Vacant(m) = this.queries.entry(input) {
                        m.insert(QueryStateValue::Completed { id, result });
                    }
                }
            }
        }

        self
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

impl<T> Clone for QueryState<T>
where
    T: Query + 'static,
{
    fn clone(&self) -> Self {
        Self {
            ctr: self.ctr,
            queries: self.queries.clone(),
        }
    }
}

impl<T> WithNotion<Deferred<RunQuery<T>>> for QueryState<T>
where
    T: Query + 'static,
{
    fn apply(mut self: Rc<Self>, notion: Rc<Deferred<RunQuery<T>>>) -> Rc<Self> {
        match *notion {
            Deferred::Pending { ref input } => {
                let RunQueryInput {
                    input,
                    id,
                    is_refresh,
                    ..
                } = input.as_ref().clone();

                if let Some(m) = self.clone().queries.get(&input) {
                    // Only mark refresh requests as outdated as other requests are marked in different places.
                    if is_refresh {
                        // If previous state is completed, we mark current request as outdated.
                        if let QueryStateValue::Completed { result, .. } = m {
                            let this = Rc::make_mut(&mut self);
                            this.ctr += 1;

                            this.queries.insert(
                                input,
                                QueryStateValue::Outdated {
                                    id,
                                    result: result.clone(),
                                },
                            );
                        }
                    }

                    return self;
                }

                let this = Rc::make_mut(&mut self);
                this.ctr += 1;

                this.queries.insert(input, QueryStateValue::Loading { id });
            }
            Deferred::Completed {
                ref input,
                ref output,
            } => {
                let RunQueryInput { input, id, .. } = input.as_ref().clone();
                if let Some(ref output) = output.as_ref() {
                    let this = Rc::make_mut(&mut self);
                    this.ctr += 1;

                    this.queries.insert(
                        input,
                        QueryStateValue::Completed {
                            id,
                            result: output.clone(),
                        },
                    );
                }
            }
            Deferred::Outdated { ref input } => {
                let RunQueryInput { input, id, .. } = input.as_ref().clone();
                if let Some(QueryStateValue::Completed {
                    id: current_id,
                    result: current_result,
                }) = self.queries.get(&input).cloned()
                {
                    if current_id == id {
                        let this = Rc::make_mut(&mut self);
                        this.ctr += 1;

                        this.queries.insert(
                            input.clone(),
                            QueryStateValue::Outdated {
                                id,
                                result: current_result,
                            },
                        );
                    }
                }
            }
        }

        self
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
