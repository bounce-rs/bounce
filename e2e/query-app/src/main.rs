use std::convert::Infallible;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::prelude::*;
use bounce::query::{use_prepared_query, use_query, Query, QueryResult};
use bounce::BounceRoot;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Debug, PartialEq)]
struct TestQuery(String);

#[async_trait(?Send)]
impl Query for TestQuery {
    type Input = u32;
    type Error = Infallible;

    async fn query(_states: &BounceStates, input: Rc<u32>) -> QueryResult<Self> {
        let url = format!("http://localhost:8081/get?n={}", *input);
        let resp = Request::get(&url).send().await.unwrap();
        let body = resp.text().await.unwrap();
        Ok(TestQuery(body).into())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PreparedTestQuery(String);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PreparedQueryError(String);

impl std::fmt::Display for PreparedQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PreparedQueryError {}

#[async_trait(?Send)]
impl Query for PreparedTestQuery {
    type Input = u32;
    type Error = PreparedQueryError;

    async fn query(_states: &BounceStates, input: Rc<u32>) -> QueryResult<Self> {
        let url = format!("http://localhost:8081/get?n={}", *input);
        let resp = Request::get(&url)
            .send()
            .await
            .map_err(|e| PreparedQueryError(e.to_string()))?;
        let body = resp
            .text()
            .await
            .map_err(|e| PreparedQueryError(e.to_string()))?;
        Ok(PreparedTestQuery(body).into())
    }
}

#[derive(Properties, PartialEq)]
struct QueryDisplayProps {
    input: u32,
}

#[function_component]
fn QueryDisplay(props: &QueryDisplayProps) -> HtmlResult {
    let result = use_query::<TestQuery>(props.input.into())?;
    Ok(html! {
        <div id="query-result" data-input={props.input.to_string()}>
            { format!("{:?}", result.as_ref().map(|q| &q.0)) }
        </div>
    })
}

#[function_component]
fn PreparedDisplay(props: &QueryDisplayProps) -> HtmlResult {
    let result = use_prepared_query::<PreparedTestQuery>(props.input.into())?;
    Ok(html! {
        <div id="prepared-result" data-input={props.input.to_string()}>
            { format!("{:?}", result.as_ref().map(|q| &q.0)) }
        </div>
    })
}

#[function_component]
fn QuerySection() -> Html {
    let counter = use_state(|| 0u32);

    html! {
        <div>
            <button id="query-prev" onclick={let counter = counter.clone(); move |_: MouseEvent| {
                if *counter > 0 { counter.set(*counter - 1); }
            }}>{"Prev"}</button>
            <span id="query-counter">{*counter}</span>
            <button id="query-next" onclick={let counter = counter.clone(); move |_: MouseEvent| {
                counter.set(*counter + 1)
            }}>{"Next"}</button>
            <Suspense fallback={html! { <div id="query-loading">{"Loading..."}</div> }}>
                <QueryDisplay input={*counter} />
            </Suspense>
        </div>
    }
}

#[function_component]
fn PreparedSection() -> Html {
    let counter = use_state(|| 0u32);

    html! {
        <div>
            <button id="prepared-prev" onclick={let counter = counter.clone(); move |_: MouseEvent| {
                if *counter > 0 { counter.set(*counter - 1); }
            }}>{"Prev"}</button>
            <span id="prepared-counter">{*counter}</span>
            <button id="prepared-next" onclick={let counter = counter.clone(); move |_: MouseEvent| {
                counter.set(*counter + 1)
            }}>{"Next"}</button>
            <Suspense fallback={html! { <div id="prepared-loading">{"Loading..."}</div> }}>
                <PreparedDisplay input={*counter} />
            </Suspense>
        </div>
    }
}

#[function_component]
fn App() -> Html {
    let mode = use_state(|| "query".to_string());

    {
        let mode = mode.clone();
        use_effect_with((), move |_| {
            let hash = web_sys::window()
                .and_then(|w| w.location().hash().ok())
                .unwrap_or_default();
            if hash == "#prepared" {
                mode.set("prepared".to_string());
            }
            || {}
        });
    }

    html! {
        <BounceRoot>
            if *mode == "query" {
                <QuerySection />
            } else {
                <PreparedSection />
            }
        </BounceRoot>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
