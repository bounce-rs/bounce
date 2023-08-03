use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use yew::platform::pinned::oneshot;
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

#[derive(PartialEq, Debug)]
pub(super) enum MutationSliceValue<T>
where
    T: Mutation + 'static,
{
    Idle,
    Loading {
        id: MutationId,
    },
    Completed {
        id: MutationId,
        result: MutationResult<T>,
    },
    Outdated {
        id: MutationId,
        result: MutationResult<T>,
    },
}

impl<T> Clone for MutationSliceValue<T>
where
    T: Mutation + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Loading { id } => Self::Loading { id: *id },
            Self::Completed { id, result } => Self::Completed {
                id: *id,
                result: result.clone(),
            },
            Self::Outdated { id, result } => Self::Outdated {
                id: *id,
                result: result.clone(),
            },
        }
    }
}

pub(super) struct RunMutationInput<T>
where
    T: Mutation,
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

pub(super) enum MutationSliceAction {
    /// Start tracking a handle.
    Create(HandleId),
    /// Stop tracking a handle.
    Destroy(HandleId),
}

#[derive(Slice, Debug)]
#[bounce(with_notion(Deferred<RunMutation<T>>))]
pub(super) struct MutationSlice<T>
where
    T: Mutation + 'static,
{
    ctr: u64,
    mutations: HashMap<HandleId, MutationSliceValue<T>>,
}

impl<T> PartialEq for MutationSlice<T>
where
    T: Mutation + 'static,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.ctr == rhs.ctr
    }
}

impl<T> Default for MutationSlice<T>
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

impl<T> Clone for MutationSlice<T>
where
    T: Mutation + 'static,
{
    fn clone(&self) -> Self {
        Self {
            ctr: self.ctr,
            mutations: self.mutations.clone(),
        }
    }
}

impl<T> Reducible for MutationSlice<T>
where
    T: Mutation + 'static,
{
    type Action = MutationSliceAction;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        {
            let this = Rc::make_mut(&mut self);
            // we don't increase the counter here as there's nothing to update.

            match action {
                Self::Action::Create(id) => {
                    this.mutations.insert(id, MutationSliceValue::Idle);
                }

                Self::Action::Destroy(id) => {
                    this.mutations.remove(&id);
                }
            }
        }

        self
    }
}

impl<T> WithNotion<Deferred<RunMutation<T>>> for MutationSlice<T>
where
    T: Mutation + 'static,
{
    fn apply(mut self: Rc<Self>, notion: Rc<Deferred<RunMutation<T>>>) -> Rc<Self> {
        match notion.as_ref() {
            Deferred::Completed {
                ref input,
                ref output,
            } => {
                let this = Rc::make_mut(&mut self);
                this.ctr += 1;

                match this.mutations.entry(input.handle_id) {
                    Entry::Vacant(_m) => {
                        return self; // The handle has been destroyed so there's no need to track it any more.
                    }
                    Entry::Occupied(mut m) => {
                        let m = m.get_mut();
                        match m {
                            MutationSliceValue::Loading { id }
                            | MutationSliceValue::Completed { id, .. }
                            | MutationSliceValue::Outdated { id, .. } => {
                                // only replace if new id is higher.
                                if *id <= input.mutation_id {
                                    *m = MutationSliceValue::Completed {
                                        id: input.mutation_id,
                                        result: output.as_ref().clone(),
                                    };
                                }
                            }
                            MutationSliceValue::Idle => {
                                *m = MutationSliceValue::Completed {
                                    id: input.mutation_id,
                                    result: output.as_ref().clone(),
                                };
                            }
                        }
                    }
                }
            }
            Deferred::Pending { input } => {
                let this = Rc::make_mut(&mut self);
                this.ctr += 1;

                match this.mutations.entry(input.handle_id) {
                    Entry::Vacant(_m) => {
                        return self; // The handle has been destroyed so there's no need to track it any more.
                    }
                    Entry::Occupied(mut m) => {
                        let m = m.get_mut();
                        match m {
                            MutationSliceValue::Loading { .. } => {}
                            MutationSliceValue::Completed { id, result } => {
                                *m = MutationSliceValue::Outdated {
                                    id: *id,
                                    result: result.clone(),
                                };
                            }
                            MutationSliceValue::Outdated { .. } => {}
                            MutationSliceValue::Idle => {
                                *m = MutationSliceValue::Loading {
                                    id: input.mutation_id,
                                };
                            }
                        }
                    }
                }
            }
            Deferred::Outdated { .. } => {}
        }

        self
    }
}

#[derive(PartialEq)]
pub(super) struct MutationSelector<T>
where
    T: Mutation + 'static,
{
    pub id: Option<MutationId>,
    pub value: Option<MutationSliceValue<T>>,
}

impl<T> InputSelector for MutationSelector<T>
where
    T: Mutation + 'static,
{
    type Input = HandleId;
    fn select(states: &BounceStates, input: Rc<HandleId>) -> Rc<Self> {
        let value = states
            .get_slice_value::<MutationSlice<T>>()
            .mutations
            .get(&input)
            .cloned();

        let id = value.as_ref().and_then(|m| match m {
            MutationSliceValue::Loading { id }
            | MutationSliceValue::Completed { id, .. }
            | MutationSliceValue::Outdated { id, .. } => Some(*id),
            MutationSliceValue::Idle => None,
        });

        Self { id, value }.into()
    }
}
