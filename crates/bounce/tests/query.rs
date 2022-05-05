#![cfg(feature = "query")]

use std::convert::Infallible;
use std::rc::Rc;
use std::time::Duration;

use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

use async_trait::async_trait;
use bounce::prelude::*;
use bounce::query::{use_query_value, Query, QueryResult};
use bounce::BounceRoot;
use gloo::timers::future::sleep;
use gloo::utils::document;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

async fn get_text_content<S: AsRef<str>>(selector: S) -> String {
    sleep(Duration::ZERO).await;

    document()
        .query_selector(selector.as_ref())
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap()
}

#[test]
async fn test_query_requery_upon_state_change() {
    #[derive(PartialEq, Default, Atom)]
    pub struct MyState {
        inner: usize,
    }

    #[derive(PartialEq, Default)]
    pub struct MyQuery {
        inner: usize,
    }

    #[async_trait(?Send)]
    impl Query for MyQuery {
        type Input = ();
        type Error = Infallible;

        async fn query(states: &BounceStates, _input: Rc<()>) -> QueryResult<Self> {
            let inner = states.get_atom_value::<MyState>().inner;

            sleep(Duration::ZERO).await;

            Ok(MyQuery { inner }.into())
        }
    }

    #[function_component(Comp)]
    fn comp() -> Html {
        let my_query = use_query_value::<MyQuery>(().into());
        let set_my_state = use_atom_setter();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    sleep(Duration::from_millis(50)).await;

                    set_my_state(MyState { inner: 1 });
                });

                || {}
            },
            (),
        );

        match my_query.result() {
            None => {
                html! { <div id="content">{"Loading..."}</div> }
            }
            Some(Ok(m)) => {
                html! { <div id="content">{format!("value: {}", m.inner)}</div> }
            }
            Some(Err(_)) => unreachable!(),
        }
    }

    #[function_component(App)]
    fn app() -> Html {
        html! {
            <BounceRoot>
                <Comp />
            </BounceRoot>
        }
    }

    yew::start_app_in_element::<App>(document().query_selector("#output").unwrap().unwrap());

    let s = get_text_content("#content").await;
    assert_eq!(s, "Loading...");

    let s = get_text_content("#content").await;
    assert_eq!(s, "value: 0");

    sleep(Duration::from_millis(100)).await;

    let s = get_text_content("#content").await;
    assert_eq!(s, "value: 1");
}
