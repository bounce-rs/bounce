use std::cell::RefCell;
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
    Completed { id: Id, result: QueryResult<T> },
    Outdated((Id, QueryResult<T>)),
}

impl<T> QueryStateValue<T>
where
    T: Query + 'static,
{
    pub(crate) fn id(&self) -> Id {
        match self {
            Self::Loading(ref id) => *id,
            Self::Completed { ref id, .. } => *id,
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
            Self::Completed { id, ref result } => Self::Completed {
                id: *id,
                result: result.clone(),
            },
            Self::Outdated(ref m) => Self::Outdated(m.clone()),
        }
    }
}

pub(super) enum QueryStateAction<T>
where
    T: Query + 'static,
{
    Refresh(Rc<(Id, Rc<T::Input>)>),
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
            Self::Action::Refresh(input) => {
                let (id, input) = (*input).clone();

                if self.queries.get(&input).map(|m| m.id()) == Some(id) {
                    let this = Rc::make_mut(&mut self);
                    this.ctr += 1;

                    this.queries.remove(&input);
                }

                self
            }

            Self::Action::LoadPrepared { id, input, result } => {
                if self.queries.get(&input).is_none() {
                    let this = Rc::make_mut(&mut self);
                    this.ctr += 1;

                    this.queries
                        .insert(input, QueryStateValue::Completed { id, result });
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
                    queries.insert(
                        input,
                        QueryStateValue::Completed {
                            id,
                            result: (*output).clone(),
                        },
                    );

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
                if let Some(QueryStateValue::Completed {
                    id: ref current_id,
                    result: current_result,
                }) = self.queries.get(&input)
                {
                    if *current_id == id {
                        let mut queries = self.queries.clone();
                        queries.insert(
                            input.clone(),
                            QueryStateValue::Outdated((id, current_result.clone())),
                        );

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
