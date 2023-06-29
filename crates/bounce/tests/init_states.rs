use std::time::Duration;

use anymap2::AnyMap;
use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

use bounce::prelude::*;
use bounce::BounceRoot;
use gloo::timers::future::sleep;
use gloo::utils::document;
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

#[derive(Atom, PartialEq, Default)]
struct State
{
    inner: u32,
}

#[function_component(Comp)]
fn comp() -> Html {
    let a = use_atom_value::<State>();

    html! {
        <div>
            <div id="a">{a.inner}</div>
        </div>
    }
}

#[test]
async fn test_without_init_states() {
    #[function_component(Root)]
    fn root() -> Html {
        html! {
            <BounceRoot>
                <Comp />
            </BounceRoot>
        }
    }

    yew::Renderer::<Root>::with_root(document().query_selector("#output").unwrap().unwrap())
        .render();

    let s = get_text_content("#a").await;
    assert_eq!(s, "0");
}

#[test]
async fn test_with_init_states() {
    #[function_component(Root)]
    fn root() -> Html {
        fn get_init_states(_: ()) -> AnyMap {
            let mut map = AnyMap::new();
            map.insert(State{ inner: 1 });

            map
        }

        html! {
            <BounceRoot {get_init_states}>
                <Comp />
            </BounceRoot>
        }
    }

    yew::Renderer::<Root>::with_root(document().query_selector("#output").unwrap().unwrap())
        .render();

    let s = get_text_content("#a").await;
    assert_eq!(s, "1");
}
