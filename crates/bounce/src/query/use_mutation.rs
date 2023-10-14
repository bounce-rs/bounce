use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use yew::platform::pinned::oneshot;
use yew::prelude::*;

use super::traits::{Mutation, MutationResult};
use crate::states::future_notion::{use_future_notion_runner, FutureNotion};
use crate::states::input_selector::use_input_selector_value;
use crate::states::slice::use_slice_dispatch;

use super::mutation_states::{
    HandleId, MutationId, MutationSelector, MutationSlice, MutationSliceAction, MutationSliceValue,
    RunMutation, RunMutationInput,
};

/// Mutation State
#[derive(Debug, PartialEq)]
pub enum MutationState<T>
where
    T: Mutation + 'static,
{
    /// The mutation has not started yet.
    Idle,
    /// The mutation is loading.
    Loading,
    /// The mutation has completed.
    Completed {
        /// Result of the completed mutation.
        result: MutationResult<T>,
    },
    /// A previous mutation has completed and a new mutation is currently loading.
    Refreshing {
        /// Result of last completed mutation.
        last_result: MutationResult<T>,
    },
}

impl<T> Clone for MutationState<T>
where
    T: Mutation + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Loading => Self::Loading,
            Self::Completed { result } => Self::Completed {
                result: result.clone(),
            },
            Self::Refreshing { last_result } => Self::Refreshing {
                last_result: last_result.clone(),
            },
        }
    }
}

impl<T> PartialEq<&MutationState<T>> for MutationState<T>
where
    T: Mutation + 'static,
{
    fn eq(&self, other: &&MutationState<T>) -> bool {
        self == *other
    }
}

impl<T> PartialEq<MutationState<T>> for &'_ MutationState<T>
where
    T: Mutation + 'static,
{
    fn eq(&self, other: &MutationState<T>) -> bool {
        *self == other
    }
}

/// A handle returned by [`use_mutation`].
pub struct UseMutationHandle<T>
where
    T: Mutation + 'static,
{
    id: HandleId,
    state: Rc<MutationState<T>>,
    run_mutation: Rc<dyn Fn(<RunMutation<T> as FutureNotion>::Input)>,
    _marker: PhantomData<T>,
}

impl<T> UseMutationHandle<T>
where
    T: Mutation + 'static,
{
    /// Returns the state of current mutation.
    pub fn state(&self) -> &MutationState<T> {
        self.state.as_ref()
    }

    /// Returns the result of last finished mutation (if any).
    ///
    /// - `None` indicates that a mutation is currently loading or has yet to start(idling).
    /// - `Some(Ok(m))` indicates that the last mutation is successful and the content is stored in `m`.
    /// - `Some(Err(e))` indicates that the last mutation has failed and the error is stored in `e`.
    pub fn result(&self) -> Option<&MutationResult<T>> {
        match self.state() {
            MutationState::Idle | MutationState::Loading => None,
            MutationState::Completed { result }
            | MutationState::Refreshing {
                last_result: result,
            } => Some(result),
        }
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

impl<T> fmt::Debug for UseMutationHandle<T>
where
    T: Mutation + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseMutationHandle")
            .field("state", &self.state)
            .finish()
    }
}

impl<T> Clone for UseMutationHandle<T>
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
/// use bounce::query::{Mutation, MutationResult, use_mutation, MutationState};
/// use yew::prelude::*;
/// use async_trait::async_trait;
/// use yew::platform::spawn_local;
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
///     let update_user = use_mutation::<UpdateUserMutation>();
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
///         None => if update_user.state() == MutationState::Idle {
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
#[hook]
pub fn use_mutation<T>() -> UseMutationHandle<T>
where
    T: Mutation + 'static,
{
    let id = *use_memo((), |_| HandleId::default());
    let dispatch_state = use_slice_dispatch::<MutationSlice<T>>();
    let run_mutation = use_future_notion_runner::<RunMutation<T>>();
    let state = use_input_selector_value::<MutationSelector<T>>(id.into());

    {
        use_effect_with(id, |id| {
            let id = *id;
            dispatch_state(MutationSliceAction::Create(id));

            move || {
                dispatch_state(MutationSliceAction::Destroy(id));
            }
        });
    }

    let state = use_memo(state, |state| match state.value.as_ref() {
        Some(MutationSliceValue::Idle) | None => MutationState::Idle,
        Some(MutationSliceValue::Loading { .. }) => MutationState::Loading,
        Some(MutationSliceValue::Completed { result, .. }) => MutationState::Completed {
            result: result.clone(),
        },
        Some(MutationSliceValue::Outdated { result, .. }) => MutationState::Refreshing {
            last_result: result.clone(),
        },
    });

    UseMutationHandle {
        id,
        state,
        run_mutation,
        _marker: PhantomData,
    }
}
