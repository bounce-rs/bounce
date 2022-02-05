use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use async_trait::async_trait;
use futures::channel::oneshot;
use yew::prelude::*;

use super::status::QueryStatus;
use crate::future_notion;
use crate::future_notion::{use_future_notion_runner, Deferred, FutureNotion};
use crate::input_selector::{use_input_selector_value, InputSelector};
use crate::root_state::BounceStates;
use crate::slice::{use_slice_dispatch, Slice};
use crate::utils::Id;
use crate::with_notion::WithNotion;

/// A Result returned by mutations.
pub type MutationResult<T> = std::result::Result<Rc<T>, <T as Mutation>::Error>;

/// A trait to be implemented on mutations.
///
/// # Note
///
/// This trait is implemented with [async_trait](macro@async_trait), you should apply an `#[async_trait(?Send)]`
/// attribute to your implementation of this trait.
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

struct RunMutationInput<T>
where
    T: Mutation + 'static,
{
    id: Id,
    input: Rc<T::Input>,
    sender: RefCell<Option<oneshot::Sender<MutationResult<T>>>>,
}

#[future_notion(RunMutation)]
async fn run_mutation<T>(states: &BounceStates, input: &RunMutationInput<T>) -> MutationResult<T>
where
    T: Mutation + 'static,
{
    let result = T::run(states, input.input.clone()).await;

    if let Some(m) = input.sender.borrow_mut().take() {
        let _result = m.send(result.clone());
    }

    result
}

enum MutationStateAction {
    /// Destroy all states of a hook.
    Destroy(Id),
}

#[derive(Slice)]
struct MutationState<T>
where
    T: Mutation + 'static,
{
    ctr: u64,
    mutations: HashMap<Id, Option<(Id, MutationResult<T>)>>,
}

impl<T> PartialEq for MutationState<T>
where
    T: Mutation + 'static,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.ctr == rhs.ctr
    }
}

impl<T> Default for MutationState<T>
where
    T: Mutation + 'static,
{
    fn default() -> Self {
        Self {
            ctr: 0,
            mutations: HashMap::new(),
        }
    }
}

impl<T> Reducible for MutationState<T>
where
    T: Mutation + 'static,
{
    type Action = MutationStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Destroy(id) => {
                let mut mutations = self.mutations.clone();
                mutations.remove(&id);

                Self {
                    // we don't increase the counter here as there's nothing to update.
                    ctr: self.ctr,
                    mutations,
                }
                .into()
            }
        }
    }
}

impl<T> WithNotion<Deferred<RunMutation<T>>> for MutationState<T>
where
    T: Mutation + 'static,
{
    fn apply(self: Rc<Self>, notion: Rc<Deferred<RunMutation<T>>>) -> Rc<Self> {
        match *notion {
            Deferred::Pending { ref input } => {
                if self.mutations.contains_key(&input.id) {
                    return self;
                }

                let mut mutations = self.mutations.clone();
                mutations.insert(input.id, None);

                Self {
                    ctr: self.ctr + 1,
                    mutations,
                }
                .into()
            }
            Deferred::Completed {
                ref input,
                ref output,
            } => {
                let mut mutations = self.mutations.clone();
                match mutations.entry(input.id) {
                    Entry::Vacant(m) => {
                        m.insert(Some((input.id, (**output).clone())));
                    }
                    Entry::Occupied(mut m) => {
                        let m = m.get_mut();
                        match m {
                            Some(ref n) => {
                                // only replace if new id is higher.
                                if n.0 <= input.id {
                                    *m = Some((input.id, (**output).clone()));
                                }
                            }
                            None => {
                                *m = Some((input.id, (**output).clone()));
                            }
                        }
                    }
                }

                Self {
                    ctr: self.ctr + 1,
                    mutations,
                }
                .into()
            }
            Deferred::Outdated { .. } => self,
        }
    }
}

/// A handle returned by [`use_mutation_value`].
pub struct UseMutationValueHandle<T>
where
    T: Mutation + 'static,
{
    state: Rc<MutationSelector<T>>,
    run_mutation: Rc<dyn Fn(<RunMutation<T> as FutureNotion>::Input)>,
    _marker: PhantomData<T>,
}

impl<T> UseMutationValueHandle<T>
where
    T: Mutation + 'static,
{
    /// Returns the status of current mutation.
    pub fn status(&self) -> QueryStatus {
        match self.state.value {
            Some(Some(Ok(_))) => QueryStatus::Ok,
            Some(Some(Err(_))) => QueryStatus::Err,
            Some(None) => QueryStatus::Loading,
            None => QueryStatus::Idle,
        }
    }

    /// Returns the result of last finished mutation (if any).
    pub fn result(&self) -> Option<MutationResult<T>> {
        self.state.value.clone().flatten()
    }

    /// Runs a mutation with input.
    pub async fn run(&self, input: impl Into<Rc<T::Input>>) -> MutationResult<T> {
        let id = Id::new();
        let input = input.into();
        let (sender, receiver) = oneshot::channel();

        (self.run_mutation)(RunMutationInput {
            id,
            input,
            sender: Some(sender).into(),
        });

        receiver.await.unwrap()
    }
}

impl<T> fmt::Debug for UseMutationValueHandle<T>
where
    T: Mutation + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseMutationValueHandle")
            .field("state", &self.state.value)
            .finish()
    }
}

#[derive(PartialEq)]
pub(crate) struct MutationSelector<T>
where
    T: Mutation + 'static,
{
    id: Option<Id>,
    value: Option<Option<MutationResult<T>>>,
}

impl<T> InputSelector for MutationSelector<T>
where
    T: Mutation + 'static,
{
    type Input = Id;
    fn select(states: &BounceStates, input: Rc<Id>) -> Rc<Self> {
        let values = states
            .get_slice_value::<MutationState<T>>()
            .mutations
            .get(&input)
            .map(|m| m.as_ref().cloned());

        let id = values.clone().flatten().map(|m| m.0);

        Self {
            id,
            value: values.map(|m| m.map(|m| m.1)),
        }
        .into()
    }
}

/// A hook to run a mutation and subscribes to its result.
///
/// A mutation is a state that is not started until the run method is invoked. Mutations are
/// usually used to modify data on the server.
pub fn use_mutation_value<T>() -> UseMutationValueHandle<T>
where
    T: Mutation + 'static,
{
    let id = *use_ref(Id::new);
    let dispatch_state = use_slice_dispatch::<MutationState<T>>();
    let state = use_input_selector_value::<MutationSelector<T>>(id.into());
    let run_mutation = use_future_notion_runner::<RunMutation<T>>();

    {
        use_effect_with_deps(
            |id| {
                let id = *id;
                move || {
                    dispatch_state(MutationStateAction::Destroy(id));
                }
            },
            id,
        );
    }

    UseMutationValueHandle {
        state,
        run_mutation,
        _marker: PhantomData,
    }
}
