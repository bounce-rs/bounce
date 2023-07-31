use std::rc::Rc;

use async_trait::async_trait;
use bounce::prelude::*;
use bounce::query::{
    use_mutation, use_query, use_query_value, Mutation, MutationResult, Query, QueryResult,
    QueryStatus,
};
use bounce::BounceRoot;
use log::Level;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use uuid::Uuid;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew::TargetCast;

#[cfg(not(test))]
static UUID_PATH: &str = "https://httpbin.org/uuid";
#[cfg(not(test))]
static ECHO_PATH: &str = "https://httpbin.org/anything";

#[cfg(test)]
static UUID_PATH: &str = "http://localhost:8080/uuid";
#[cfg(test)]
static ECHO_PATH: &str = "http://localhost:8080/anything";

#[derive(PartialEq, Debug, Serialize, Deserialize, Eq)]
struct EchoInput {
    content: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Eq)]
struct EchoMutation {
    content: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Eq)]
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
            .post(ECHO_PATH)
            .json(&*input)
            .send()
            .await
            .unwrap();
        let uuid_resp = resp.json::<EchoResponse>().await.unwrap();

        Ok(uuid_resp.json.into())
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Eq)]
struct UuidQuery {
    uuid: Uuid,
}

#[async_trait(?Send)]
impl Query for UuidQuery {
    type Input = ();
    type Error = Infallible;

    async fn query(_states: &BounceStates, _input: Rc<()>) -> QueryResult<Self> {
        // errors should be handled properly in actual application.
        let resp = reqwest::get(UUID_PATH).await.unwrap();
        let uuid_resp = resp.json::<UuidQuery>().await.unwrap();

        Ok(uuid_resp.into())
    }
}

#[derive(Debug, Properties, PartialEq, Eq)]
struct ContentProps {
    ord: usize,
}

#[function_component(Content)]
fn content(props: &ContentProps) -> Html {
    let uuid_state = use_query_value::<UuidQuery>(().into());

    let text = match (uuid_state.result(), uuid_state.status()) {
        (Some(Ok(m)), QueryStatus::Refreshing) => {
            format!("Refreshing... Last Random UUID: {}", m.uuid)
        }
        (Some(Ok(m)), _) => format!("Random UUID: {}", m.uuid),
        (Some(Err(_)), _) => unreachable!(),
        (None, _) => "Loading UUID, Please wait...".to_string(),
    };

    html! {
        <>
            <div id={format!("query-content-{}", props.ord)}>{text}</div>
        </>
    }
}

#[function_component(SuspendContent)]
fn suspend_content(props: &ContentProps) -> HtmlResult {
    let uuid_state = use_query::<UuidQuery>(().into())?;

    let text = match (uuid_state.as_deref(), uuid_state.status()) {
        (Ok(m), QueryStatus::Refreshing) => format!("Refreshing... Last Random UUID: {}", m.uuid),
        (Ok(m), _) => format!("Random UUID: {}", m.uuid),
        (Err(_), _) => unreachable!(),
    };

    Ok(html! {
        <>
            <div id={format!("query-content-{}", props.ord)}>{text}</div>
        </>
    })
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
            <button {disabled} onclick={on_fetch_clicked} id="query-refresh">{"Fetch"}</button>
        </>
    }
}

#[function_component(Echo)]
fn echo() -> Html {
    let echo_state = use_mutation::<EchoMutation>();
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
            <div id="mut-resp">{resp}</div>
            <input id="mut-input" type="text" {disabled} placeholder="Enter Content Here..." oninput={on_input} value={value.to_string()} />
            <button id="mut-submit" {disabled} onclick={on_send_clicked}>{"Send"}</button>
        </>
    }
}

#[function_component(App)]
fn app() -> Html {
    let fallback = html! {
        <>
            <div id={"query-content-0"}>{"Loading UUID, Please wait..."}</div>
        </>
    };

    html! {
        <BounceRoot>
            <h1>{"Query"}</h1>
            <div>{"When content is loading, only 1 request will be sent for the same input."}</div>
            <Suspense {fallback}>
                <SuspendContent ord={0} />
            </Suspense>
            <Content ord={1} />
            <Refresher />
            <h1>{"Mutation"}</h1>
            <Echo />
        </BounceRoot>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
mod tests {
    use super::*;
    use gloo::timers::future::sleep;
    use gloo::utils::document;
    use std::time::Duration;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use web_sys::{EventTarget, HtmlElement};

    wasm_bindgen_test_configure!(run_in_browser);

    async fn get_text_content_by_id<S: AsRef<str>>(id: S) -> String {
        sleep(Duration::ZERO).await;

        document()
            .query_selector(&format!("#{}", id.as_ref()))
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap()
    }

    async fn click_by_id<S: AsRef<str>>(id: S) {
        sleep(Duration::ZERO).await;

        document()
            .query_selector(&format!("#{}", id.as_ref()))
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlElement>()
            .click();
    }

    #[wasm_bindgen_test]
    async fn test_query_mutation() {
        yew::Renderer::<App>::with_root(document().query_selector("#output").unwrap().unwrap())
            .render();

        assert_eq!(
            get_text_content_by_id("query-content-0").await,
            "Loading UUID, Please wait..."
        );
        assert_eq!(
            get_text_content_by_id("query-content-1").await,
            "Loading UUID, Please wait..."
        );

        let mut found = false;
        for _i in 0..1000 {
            sleep(Duration::from_millis(100)).await;

            if get_text_content_by_id("query-content-0")
                .await
                .starts_with("Random UUID: ")
            {
                // ensure only 1 request is sent.
                assert_eq!(
                    get_text_content_by_id("query-content-0").await,
                    get_text_content_by_id("query-content-1").await
                );

                found = true;

                break;
            }
        }

        assert!(found, "request didn't finish in time!");

        let uuid_found = get_text_content_by_id("query-content-0").await;
        click_by_id("query-refresh").await;

        let mut found = false;
        for _i in 0..1000 {
            sleep(Duration::from_millis(10)).await;

            if get_text_content_by_id("query-content-0")
                .await
                .starts_with("Refreshing...")
            {
                // make sure uuid hasn't changed.
                assert_eq!(
                    format!("Refreshing... Last {}", uuid_found),
                    get_text_content_by_id("query-content-0").await
                );

                // ensure only 1 request is sent.
                assert_eq!(
                    get_text_content_by_id("query-content-0").await,
                    get_text_content_by_id("query-content-1").await
                );

                found = true;

                break;
            }
        }

        assert!(found, "request didn't show as refetching!");

        let mut found = false;
        for _i in 0..1000 {
            sleep(Duration::from_millis(100)).await;

            if get_text_content_by_id("query-content-0")
                .await
                .starts_with("Random UUID: ")
            {
                // assert uuid changed.
                assert_ne!(uuid_found, get_text_content_by_id("query-content-0").await);

                // ensure only 1 request is sent.
                assert_eq!(
                    get_text_content_by_id("query-content-0").await,
                    get_text_content_by_id("query-content-1").await
                );

                found = true;

                break;
            }
        }

        assert!(found, "request didn't finish in time!");

        document()
            .query_selector("#mut-input")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .set_value("some content");

        document()
            .query_selector("#mut-input")
            .unwrap()
            .unwrap()
            .unchecked_into::<EventTarget>()
            .dispatch_event(&Event::new("input").unwrap())
            .unwrap();

        assert_eq!(
            get_text_content_by_id("mut-resp").await,
            "To send the content to server, please click 'Send'."
        );

        click_by_id("mut-submit").await;

        let mut found = false;
        for _i in 0..100 {
            sleep(Duration::from_millis(100)).await;

            if get_text_content_by_id("mut-resp")
                .await
                .starts_with("Server Response: ")
            {
                // ensure only 1 request is sent.
                assert_eq!(
                    get_text_content_by_id("mut-resp").await,
                    "Server Response: some content"
                );

                found = true;

                break;
            }
        }

        assert!(found, "mutation didn't finish in time!");
    }
}
