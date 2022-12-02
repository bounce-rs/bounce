use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use futures::channel::oneshot;
use yew::prelude::*;

use super::traits::{Mutation, MutationResult};
use crate::future_notion;
use crate::root_state::BounceStates;
use crate::states::future_notion::Deferred;
use crate::states::input_selector::InputSelector;
use crate::states::notion::WithNotion;
use crate::states::slice::Slice;
use crate::utils::Id;

// We create 2 ID types to better distinguish them in code.
#[derive(Default, PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct HandleId(Id);

#[derive(Default, PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct MutationId(Id);

pub(super) struct RunMutationInput<T>
where
    T: Mutation + 'static,
{
    pub handle_id: HandleId,
    pub mutation_id: MutationId,
    pub input: Rc<T::Input>,
    pub sender: RefCell<Option<oneshot::Sender<MutationResult<T>>>>,
}

#[future_notion(RunMutation)]
pub(super) async fn run_mutation<T>(
    states: &BounceStates,
    input: &RunMutationInput<T>,
) -> MutationResult<T>
where
    T: Mutation + 'static,
{
    let result = T::run(states, input.input.clone()).await;

    if let Some(m) = input.sender.borrow_mut().take() {
        let _result = m.send(result.clone());
    }

    result
}

pub(super) enum MutationStateAction {
    /// Start tracking a handle.
    Create(HandleId),
    /// Stop tracking a handle.
    Destroy(HandleId),
}

#[derive(Slice, Debug)]
#[bounce(with_notion(Deferred<RunMutation<T>>))]
pub(super) struct MutationState<T>
where
    T: Mutation + 'static,
{
    ctr: u64,
    mutations: HashMap<HandleId, Option<(MutationId, MutationResult<T>)>>,
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
            Self::Action::Create(id) => {
                let mut mutations = self.mutations.clone();
                mutations.insert(id, None);

                Self {
                    // we don't increase the counter here as there's nothing to update.
                    ctr: self.ctr,
                    mutations,
                }
                .into()
            }

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
            Deferred::Completed {
                ref input,
                ref output,
            } => {
                let mut mutations = self.mutations.clone();
                match mutations.entry(input.handle_id) {
                    Entry::Vacant(_m) => {
                        return self; // The handle has been destroyed so there's no need to track it any more.
                    }
                    Entry::Occupied(mut m) => {
                        let m = m.get_mut();
                        match m {
                            Some(ref n) => {
                                // only replace if new id is higher.
                                if n.0 <= input.mutation_id {
                                    *m = Some((input.mutation_id, (**output).clone()));
                                }
                            }
                            None => {
                                *m = Some((input.mutation_id, (**output).clone()));
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
            Deferred::Pending { .. } => self,
            Deferred::Outdated { .. } => self,
        }
    }
}

#[derive(PartialEq)]
pub(super) struct MutationSelector<T>
where
    T: Mutation + 'static,
{
    pub id: Option<MutationId>,
    pub value: Option<Option<MutationResult<T>>>,
}

impl<T> InputSelector for MutationSelector<T>
where
    T: Mutation + 'static,
{
    type Input = HandleId;
    fn select(states: &BounceStates, input: Rc<HandleId>) -> Rc<Self> {
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
