//! This is a demo of the low-level future notion API.
//!
//! If you are interacting with APIs, it is recommended to use the Query API which is implemented
//! based on the future notion API with automatic caching and request deduplication.

use std::rc::Rc;

use bounce::*;
use log::Level;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Serialize, Deserialize)]
struct UuidResponse {
    uuid: String,
}

#[future_notion(FetchUuid)]
async fn fetch_uuid(_input: &()) -> String {
    // errors should be handled properly in actual application.
    let resp = reqwest::get("https://httpbin.org/uuid").await.unwrap();
    let uuid_resp = resp.json::<UuidResponse>().await.unwrap();

    uuid_resp.uuid
}

#[derive(PartialEq, Atom)]
#[bounce(with_notion(Deferred<FetchUuid>))]
enum UuidState {
    NotStarted,
    Pending,
    Complete(String),
}

impl Default for UuidState {
    fn default() -> UuidState {
        Self::NotStarted
    }
}

impl WithNotion<Deferred<FetchUuid>> for UuidState {
    fn apply(self: Rc<Self>, notion: Rc<Deferred<FetchUuid>>) -> Rc<Self> {
        match notion.output() {
            Some(m) => Self::Complete(m.to_string()).into(),
            None => Self::Pending.into(),
        }
    }
}

#[function_component(Reader)]
fn reader() -> Html {
    let uuid_state = use_atom_value::<UuidState>();

    let text = match *uuid_state {
        UuidState::NotStarted => {
            "Please click on Fetch to fetch a random UUID from remote.".to_string()
        }
        UuidState::Pending => "Loading UUID, Please wait...".to_string(),
        UuidState::Complete(ref m) => format!("Random UUID: {}", m),
    };

    html! { <div>{text}</div> }
}

#[function_component(Loader)]
fn loader() -> Html {
    let uuid_state = use_atom::<UuidState>();
    let run_fetch_uuid = use_future_notion_runner::<FetchUuid>();

    let on_fetch_clicked = Callback::from(move |_| run_fetch_uuid(()));

    let disabled = *uuid_state == UuidState::Pending;

    html! { <button {disabled} onclick={on_fetch_clicked}>{"Fetch"}</button> }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <Reader />
            <Loader />
        </BounceRoot>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::start_app::<App>();
}
