use std::rc::Rc;

use async_trait::async_trait;
use bounce::prelude::*;
use bounce::query::{use_prepared_query, Query, QueryResult};
use bounce::BounceRoot;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use yew::platform::spawn_local;
use yew::prelude::*;

#[derive(PartialEq, Debug, Serialize, Deserialize, Eq, Clone)]
struct UuidQuery {
    uuid: Uuid,
}

// To be replaced with `!` once it is stable.
#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[error("this will never happen.")]
struct Never {}

#[async_trait(?Send)]
impl Query for UuidQuery {
    type Input = ();
    type Error = Never;

    async fn query(_states: &BounceStates, _input: Rc<()>) -> QueryResult<Self> {
        // errors should be handled properly in actual application.
        let resp = reqwest::get("https://httpbin.org/uuid").await.unwrap();
        let uuid_resp = resp.json::<UuidQuery>().await.unwrap();

        Ok(uuid_resp.into())
    }
}

#[derive(Debug, Properties, PartialEq, Eq)]
struct ContentProps {
    ord: usize,
}

#[function_component(SuspendContent)]
fn suspend_content(props: &ContentProps) -> HtmlResult {
    let uuid_state = use_prepared_query::<UuidQuery>(().into())?;

    let text = match uuid_state.as_deref() {
        Ok(m) => format!("Random UUID: {}", m.uuid),
        Err(_) => unreachable!(),
    };

    Ok(html! {
        <>
            <div id={format!("query-content-{}", props.ord)}>{text}</div>
        </>
    })
}

#[function_component(Refresher)]
fn refresher() -> HtmlResult {
    let uuid_state = use_prepared_query::<UuidQuery>(().into())?;

    let on_fetch_clicked = Callback::from(move |_| {
        let uuid_state = uuid_state.clone();
        spawn_local(async move {
            let _ignore = uuid_state.refresh().await;
        });
    });

    Ok(html! {
        <>
            <button onclick={on_fetch_clicked}>{"Fetch"}</button>
        </>
    })
}

#[function_component(App)]
pub fn app() -> Html {
    let fallback = html! {
        <>
            <div id={"query-content-0"}>{"Loading UUID, Please wait..."}</div>
        </>
    };

    let fallback_refresh = html! {
        <>
            <button disabled={true}>{"Fetch"}</button>
        </>
    };

    html! {
        <BounceRoot>
            <h1>{"Query"}</h1>
            <div>{"This UUID is fetched at the server-side."}</div>
            <Suspense {fallback}>
                <SuspendContent ord={0} />
            </Suspense>
            <Suspense fallback={fallback_refresh}>
                <Refresher />
            </Suspense>
        </BounceRoot>
    }
}
