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
use crate::root_state::BounceStates;
use crate::states::future_notion::{use_future_notion_runner, Deferred, FutureNotion};
use crate::states::input_selector::{use_input_selector_value, InputSelector};
use crate::states::notion::WithNotion;
use crate::states::slice::{use_slice_dispatch, Slice};
use crate::utils::Id;

/// A Result returned by mutations.
pub type MutationResult<T> = std::result::Result<Rc<T>, <T as Mutation>::Error>;

/// A trait to be implemented on mutations.
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
/// use bounce::query::{Mutation, MutationResult};
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
/// struct UpdateUserMutation {
/// }
///
/// #[async_trait(?Send)]
/// impl Mutation for UpdateUserMutation {
///     type Input = User;
///     type Error = Infallible;
///
///     async fn run(_states: &BounceStates, _input: Rc<User>) -> MutationResult<Self> {
///         // updates the user information.
///
///         Ok(UpdateUserMutation {}.into())
///     }
/// }
/// ```
///
/// See: [`use_mutation_value`]
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

// We create 2 ID types to better distinguish them in code.
#[derive(Default, PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Ord, Copy)]
struct HandleId(Id);

#[derive(Default, PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Ord, Copy)]
struct MutationId(Id);

struct RunMutationInput<T>
where
    T: Mutation + 'static,
{
    handle_id: HandleId,
    mutation_id: MutationId,
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
    /// Start tracking a handle.
    Create(HandleId),
    /// Stop tracking a handle.
    Destroy(HandleId),
}

#[derive(Slice, Debug)]
#[bounce(with_notion(Deferred<RunMutation<T>>))]
struct MutationState<T>
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

/// A handle returned by [`use_mutation_value`].
pub struct UseMutationValueHandle<T>
where
    T: Mutation + 'static,
{
    id: HandleId,
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
    ///
    /// - `None` indicates that a mutation is currently loading or has yet to start(idling).
    /// - `Some(Ok(m))` indicates that the last mutation is successful and the content is stored in `m`.
    /// - `Some(Err(e))` indicates that the last mutation has failed and the error is stored in `e`.
    pub fn result(&self) -> Option<MutationResult<T>> {
        self.state.value.clone().flatten()
    }

    /// Runs a mutation with input.
    pub async fn run(&self, input: impl Into<Rc<T::Input>>) -> MutationResult<T> {
        let id = MutationId::default();
        let input = input.into();
        let (sender, receiver) = oneshot::channel();

        (self.run_mutation)(RunMutationInput {
            handle_id: self.id,
            mutation_id: id,
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

impl<T> Clone for UseMutationValueHandle<T>
where
    T: Mutation + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: self.state.clone(),
            run_mutation: self.run_mutation.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(PartialEq)]
struct MutationSelector<T>
where
    T: Mutation + 'static,
{
    id: Option<MutationId>,
    value: Option<Option<MutationResult<T>>>,
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

/// A hook to run a mutation and subscribes to its result.
///
/// A mutation is a state that is not started until the run method is invoked. Mutations are
/// usually used to modify data on the server.
///
/// # Example
///
/// ```
/// use std::rc::Rc;
/// use std::convert::Infallible;
/// use bounce::prelude::*;
/// use bounce::query::{Mutation, MutationResult, use_mutation_value, QueryStatus};
/// use yew::prelude::*;
/// use async_trait::async_trait;
/// use wasm_bindgen_futures::spawn_local;
///
/// #[derive(Debug, PartialEq)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// #[derive(Debug, PartialEq)]
/// struct UpdateUserMutation {
/// }
///
/// #[async_trait(?Send)]
/// impl Mutation for UpdateUserMutation {
///     type Input = User;
///     type Error = Infallible;
///
///     async fn run(_states: &BounceStates, _input: Rc<User>) -> MutationResult<Self> {
///         // updates the user information.
///
///         Ok(UpdateUserMutation {}.into())
///     }
/// }
///
/// #[function_component(Comp)]
/// fn comp() -> Html {
///     let update_user = use_mutation_value::<UpdateUserMutation>();
///
///     let on_click_update_user = {
///         let update_user = update_user.clone();
///         Callback::from(move |_| {
///             let update_user = update_user.clone();
///             spawn_local(
///                 async move {
///                     // The result is also returned to the run method, but since we will
///                     // process the result in the render function, we ignore it here.
///                     let _result = update_user.run(User {id: 0, name: "Jane Done".into() }).await;
///                 }
///             );
///         })
///     };
///
///     match update_user.result() {
///         // The result is None if the mutation is currently loading or has yet to start.
///         None => if update_user.status() == QueryStatus::Idle {
///             html! {<div>{"Updating User..."}</div>}
///         } else {
///             html! {<button onclick={on_click_update_user}>{"Updating User"}</button>}
///         },
///         // The result is Some(Ok(_)) if the mutation has succeed.
///         Some(Ok(_m)) => html! {<div>{"User has been successfully updated."}</div>},
///         // The result is Some(Err(_)) if an error is returned during fetching.
///         Some(Err(_e)) => html! {<div>{"Oops, something went wrong."}</div>},
///     }
/// }
/// ```
pub fn use_mutation_value<T>() -> UseMutationValueHandle<T>
where
    T: Mutation + 'static,
{
    let id = *use_ref(HandleId::default);
    let dispatch_state = use_slice_dispatch::<MutationState<T>>();
    let run_mutation = use_future_notion_runner::<RunMutation<T>>();
    let state = use_input_selector_value::<MutationSelector<T>>(id.into());

    {
        use_effect_with_deps(
            |id| {
                let id = *id;
                dispatch_state(MutationStateAction::Create(id));

                move || {
                    dispatch_state(MutationStateAction::Destroy(id));
                }
            },
            id,
        );
    }

    UseMutationValueHandle {
        id,
        state,
        run_mutation,
        _marker: PhantomData,
    }
}
