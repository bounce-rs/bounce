use std::rc::Rc;

use serde::de::Deserialize;
use serde::ser::Serialize;
use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

use super::query_states::{
    QuerySelector, QueryState, QueryStateAction, QueryStateValue, RunQuery, RunQueryInput,
};
use super::traits::Query;
use super::use_query::UseQueryHandle;
use crate::root_state::BounceRootState;
use crate::states::future_notion::use_future_notion_runner;
use crate::states::input_selector::use_input_selector_value;
use crate::states::slice::use_slice_dispatch;
use crate::utils::Id;

/// A hook to run a query and subscribes to its result, suspending while fetching
/// if server-side rendered values are not available.
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
/// use bounce::query::{Query, QueryResult, use_prepared_query};
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
///     let user = use_prepared_query::<UserQuery>(0.into())?;
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
pub fn use_prepared_query<T>(input: Rc<T::Input>) -> SuspensionResult<UseQueryHandle<T>>
where
    T: Query + Clone + Serialize + for<'de> Deserialize<'de> + 'static,
    T::Input: Clone + Serialize + for<'de> Deserialize<'de>,
    T::Error: Clone + Serialize + for<'de> Deserialize<'de>,
{
    let id = *use_memo(|_| Id::new(), ());
    let value_state = use_input_selector_value::<QuerySelector<T>>(input.clone());
    let dispatch_state = use_slice_dispatch::<QueryState<T>>();
    let run_query = use_future_notion_runner::<RunQuery<T>>();

    let prepared_value = {
        let _run_query = run_query.clone();
        let _root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

        let prepared_value = use_prepared_state!(
            async move |input| -> std::result::Result<T, T::Error> {
                use std::cell::RefCell;
                use std::time::Duration;

                use yew::platform::pinned::oneshot;
                use yew::platform::time::sleep;

                let (sender, receiver) = oneshot::channel();

                _run_query(RunQueryInput {
                    id,
                    input: input.clone(),
                    sender: Rc::new(RefCell::new(Some(sender))),
                });

                if let Ok(m) = receiver.await {
                    return m.map(|m| (*m).clone());
                }

                loop {
                    let states = _root.states();
                    let value_state =
                        states.get_input_selector_value::<QuerySelector<T>>(input.clone());

                    match value_state.value {
                        Some(QueryStateValue::Completed { result: ref m, .. })
                        | Some(QueryStateValue::Outdated { result: ref m, .. }) => {
                            return m.clone().map(|m| (*m).clone());
                        }
                        None | Some(QueryStateValue::Loading { .. }) => {
                            let (sender, receiver) = oneshot::channel::<()>();
                            let sender = Rc::new(RefCell::new(Some(sender)));

                            states.add_listener_callback(Rc::new(Callback::from(move |_| {
                                if let Some(m) = sender.borrow_mut().take() {
                                    let _ = m.send(());
                                }
                            })));
                            // We subscribe to the selector again.
                            states.get_input_selector_value::<QuerySelector<T>>(input.clone());

                            // We yield to event loop so state updates can be applied.
                            sleep(Duration::ZERO).await;

                            receiver.await.unwrap();
                        }
                    }
                }
            },
            (*input).clone()
        )?;

        (*use_memo(
            |p| p.clone().map(|m| (*m).clone().map(Rc::new)),
            prepared_value,
        ))
        .clone()
    };

    let value = use_memo(
        |v| match v.value {
            Some(QueryStateValue::Loading { .. }) | None => Err(Suspension::new()),
            Some(QueryStateValue::Completed { id, result: ref m })
            | Some(QueryStateValue::Outdated { id, result: ref m }) => Ok((id, m.clone())),
        },
        value_state.clone(),
    );

    {
        let input = input.clone();
        let run_query = run_query.clone();
        let dispatch_state = dispatch_state.clone();

        use_memo(
            move |_| match prepared_value {
                Some(m) => dispatch_state(QueryStateAction::LoadPrepared {
                    id,
                    input,
                    result: m,
                }),
                None => run_query(RunQueryInput {
                    id,
                    input: input.clone(),
                    sender: Rc::default(),
                }),
            },
            (),
        );
    }

    {
        let input = input.clone();
        let run_query = run_query.clone();

        use_effect_with_deps(
            move |(id, input, value_state)| {
                if matches!(value_state.value, Some(QueryStateValue::Outdated { .. })) {
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

    match value.as_ref().as_ref().cloned() {
        Ok((state_id, result)) => Ok(UseQueryHandle {
            state_id,
            input,
            dispatch_state,
            run_query,
            result,
        }),
        Err((s, _)) => Err(s.clone()),
    }
}
