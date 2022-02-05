use std::rc::Rc;

use async_trait::async_trait;
use bounce::prelude::*;
use bounce::query::{
    use_mutation_value, use_query_value, Mutation, MutationResult, Query, QueryResult, QueryStatus,
};
use bounce::BounceRoot;
use log::Level;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::TargetCast;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct EchoInput {
    content: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct EchoMutation {
    content: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct EchoResponse {
    json: EchoMutation,
}

#[async_trait(?Send)]
impl Mutation for EchoMutation {
    type Input = EchoInput;
    type Error = Infallible;

    async fn run(_states: &BounceStates, input: Rc<Self::Input>) -> MutationResult<Self> {
        // errors should be handled properly in actual application.
        let resp = reqwest::Client::new()
            .post("https://httpbin.org/anything")
            .json(&*input)
            .send()
            .await
            .unwrap();
        let uuid_resp = resp.json::<EchoResponse>().await.unwrap();

        Ok(uuid_resp.json.into())
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct UuidQuery {
    uuid: Uuid,
}

#[async_trait(?Send)]
impl Query for UuidQuery {
    type Input = ();
    type Error = Infallible;

    async fn query(_states: &BounceStates, _input: Rc<()>) -> QueryResult<Self> {
        // errors should be handled properly in actual application.
        let resp = reqwest::get("https://httpbin.org/uuid").await.unwrap();
        let uuid_resp = resp.json::<UuidQuery>().await.unwrap();

        Ok(uuid_resp.into())
    }
}

#[function_component(Content)]
fn content() -> Html {
    let uuid_state = use_query_value::<UuidQuery>(().into());

    let text = match uuid_state.result() {
        Some(Ok(m)) => format!("Random UUID: {}", m.uuid),
        Some(Err(_)) => unreachable!(),
        None => "Loading UUID, Please wait...".to_string(),
    };

    html! {
        <>
            <div>{text}</div>
        </>
    }
}

#[function_component(Refresher)]
fn refresher() -> Html {
    let uuid_state = use_query_value::<UuidQuery>(().into());

    let disabled = uuid_state.status() == QueryStatus::Loading;

    let on_fetch_clicked = Callback::from(move |_| {
        let uuid_state = uuid_state.clone();
        spawn_local(async move {
            let _ignore = uuid_state.refresh().await;
        });
    });

    html! {
        <>
            <button {disabled} onclick={on_fetch_clicked}>{"Fetch"}</button>
        </>
    }
}

#[function_component(Echo)]
fn echo() -> Html {
    let echo_state = use_mutation_value::<EchoMutation>();
    let value = use_state_eq(|| "".to_string());

    let disabled = echo_state.status() == QueryStatus::Loading;

    let on_input = {
        let value = value.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            value.set(input.value());
        })
    };

    let on_send_clicked = {
        let echo_state = echo_state.clone();
        let value = value.clone();
        Callback::from(move |_| {
            let echo_state = echo_state.clone();
            let value = value.clone();
            spawn_local(async move {
                let _ignore = echo_state
                    .run(EchoInput {
                        content: value.to_string(),
                    })
                    .await;
            });
        })
    };

    let resp = match echo_state.result() {
        Some(Ok(m)) => format!("Server Response: {}", m.content),
        Some(Err(_)) => unreachable!(),
        None => "To send the content to server, please click 'Send'.".to_string(),
    };

    html! {
        <>
            <div>{resp}</div>
            <input type="text" {disabled} placeholder="Enter Content Here..." oninput={on_input} value={value.to_string()} />
            <button {disabled} onclick={on_send_clicked}>{"Send"}</button>
        </>
    }
}

#[function_component(App)]
fn app() -> Html {
    let contents = (0..2).map(|_| html! { <Content /> }).collect::<Vec<_>>();

    html! {
        <BounceRoot>
            <h1>{"Query"}</h1>
            <div>{"When content is loading, only 1 request will be sent for the same input."}</div>
            {contents}
            <Refresher />
            <h1>{"Mutation"}</h1>
            <Echo />
        </BounceRoot>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::start_app::<App>();
}
