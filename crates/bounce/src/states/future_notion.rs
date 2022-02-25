use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use futures::future::LocalBoxFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::root_state::{BounceRootState, BounceStates};

/// A trait to implement a [`Future`](std::future::Future)-backed notion.
///
/// This trait is usually automatically implemented by the
/// [`#[future_notion]`](macro@crate::future_notion) attribute macro.
pub trait FutureNotion {
    /// The input type.
    type Input: 'static;
    /// The output type.
    type Output: 'static;

    /// Runs a future notion.
    fn run<'a>(
        states: &'a BounceStates,
        input: &'a Self::Input,
    ) -> LocalBoxFuture<'a, Self::Output>;
}

/// A deferred result type for future notions.
///
/// For each future notion `T`, a `Deferred<T>` the following notions will be applied to states:
///
/// - A `Deferred::<T>::Pending` Notion will be applied before a future notion starts running.
/// - A `Deferred::<T>::Complete` Notion will be applied after a future notion completes.
/// - If any states are used during the run of a future notion,
///   a `Deferred::<T>::Outdated` Notion will be applied **once** after the value of any used states changes.
#[derive(Debug)]
pub enum Deferred<T>
where
    T: FutureNotion,
{
    /// A future notion is running.
    Pending {
        /// The input value of a future notion.
        input: Rc<T::Input>,
    },
    /// A future notion has completed.
    Completed {
        /// The input value of a future notion.
        input: Rc<T::Input>,

        /// The output value of a future notion.
        output: Rc<T::Output>,
    },
    /// The states used in the future notion run has been changed.
    Outdated {
        /// The input value of a future notion.
        input: Rc<T::Input>,
    },
}

impl<T> Deferred<T>
where
    T: FutureNotion,
{
    /// Returns `true` if current future notion is still running.
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending { .. } => true,
            Self::Completed { .. } => false,
            Self::Outdated { .. } => false,
        }
    }

    /// Returns `true` if current future notion has been completed.
    pub fn is_completed(&self) -> bool {
        match self {
            Self::Pending { .. } => false,
            Self::Completed { .. } => true,
            Self::Outdated { .. } => false,
        }
    }

    /// Returns `true` if current future notion is outdated.
    pub fn is_outdated(&self) -> bool {
        match self {
            Self::Pending { .. } => false,
            Self::Completed { .. } => false,
            Self::Outdated { .. } => true,
        }
    }

    /// Returns the input of current future notion.
    pub fn input(&self) -> Rc<T::Input> {
        match self {
            Self::Pending { input } => input.clone(),
            Self::Completed { input, .. } => input.clone(),
            Self::Outdated { input } => input.clone(),
        }
    }

    /// Returns the output of current future notion if it has completed.
    pub fn output(&self) -> Option<Rc<T::Output>> {
        match self {
            Self::Pending { .. } => None,
            Self::Completed { output, .. } => Some(output.clone()),
            Self::Outdated { .. } => None,
        }
    }
}

impl<T> Clone for Deferred<T>
where
    T: FutureNotion,
{
    fn clone(&self) -> Self {
        match self {
            Self::Pending { ref input } => Self::Pending {
                input: input.clone(),
            },
            Self::Completed {
                ref input,
                ref output,
            } => Self::Completed {
                input: input.clone(),
                output: output.clone(),
            },
            Self::Outdated { ref input } => Self::Outdated {
                input: input.clone(),
            },
        }
    }
}

/// A hook to create a function that when called, runs a [`FutureNotion`] with provided input.
///
/// A `FutureNotion` is created by applying a `#[future_notion(NotionName)]` attribute to an async function.
///
/// When a future notion is run, it will be applied twice with a notion type [`Deferred<T>`]. The
/// first time is before it starts with a variant `Pending` and the second time is when it
/// completes with variant `Complete`.
///
/// If the notion read any other states using the `BounceStates` argument, it will subscribe to the
/// states, when any state changes, an `Outdated` variant will be dispatched.
///
/// # Note
///
/// If you are trying to interact with a backend API, it is recommended to use the [Query](crate::query) API instead.
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// # use std::fmt;
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
///
/// #[derive(PartialEq)]
/// struct User {
///     id: u64,
///     username: String,
/// }
///
/// #[future_notion(FetchUser)]
/// async fn fetch_user(id: &u64) -> User {
///     // fetch user here...
///
///     User { id: *id, username: "username".into() }
/// }
///
/// #[derive(PartialEq, Default, Atom)]
/// #[with_notion(Deferred<FetchUser>)]  // A future notion with type `T` will be applied as `Deferred<T>`.
/// struct UserState {
///     inner: Option<Rc<User>>,
/// }
///
/// // Each time a future notion is run, it will be applied twice.
/// impl WithNotion<Deferred<FetchUser>> for UserState {
///     fn apply(self: Rc<Self>, notion: Rc<Deferred<FetchUser>>) -> Rc<Self> {
///         match notion.output() {
///             Some(m) => Self { inner: Some(m) }.into(),
///             None => self,
///         }
///     }
/// }
///
/// # #[function_component(FetchUserComp)]
/// # fn fetch_user_comp() -> Html {
/// let load_user = use_future_notion_runner::<FetchUser>();
/// load_user(1);
/// # Html::default()
/// # }
/// ```
pub fn use_future_notion_runner<T>() -> Rc<dyn Fn(T::Input)>
where
    T: FutureNotion + 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    Rc::new(move |input: T::Input| {
        let root = root.clone();
        let input = Rc::new(input);

        spawn_local(async move {
            root.apply_notion(Rc::new(Deferred::<T>::Pending {
                input: input.clone(),
            }) as Rc<dyn Any>);

            let states = root.states();

            // send the listeners in to be destroyed.
            let listeners = Rc::new(RefCell::new(None));
            let listener_run = Rc::new(AtomicBool::new(false));

            {
                let listener_run = listener_run.clone();
                let listeners = listeners.clone();
                let root = root.clone();
                let input = input.clone();
                states.add_listener_callback(Rc::new(Callback::from(move |_| {
                    // There's a chance that the listeners might be called during the time while the future
                    // notion is running and there will be nothing to drop.
                    let listeners = listeners.borrow_mut().take();
                    let last_listener_run = listener_run.swap(true, Ordering::Relaxed);

                    if !last_listener_run || listeners.is_some() {
                        root.apply_notion(Rc::new(Deferred::<T>::Outdated {
                            input: input.clone(),
                        }) as Rc<dyn Any>);
                    }
                })))
            }

            let output = T::run(&states, &input).await;

            if !listener_run.load(Ordering::Relaxed) {
                let _result = listeners.borrow_mut().replace(states.take_listeners());
            }

            root.apply_notion(Rc::new(Deferred::<T>::Completed {
                input,
                output: output.into(),
            }) as Rc<dyn Any>);
        });
    })
}
