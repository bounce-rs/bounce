use std::rc::Rc;
use std::time::Duration;

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

#[test]
async fn test_notion_generic() {
    #[derive(Atom, PartialEq, Default)]
    #[bounce(with_notion(State<T>))]
    struct State<T>
    where
        T: PartialEq + Default + 'static,
    {
        inner: T,
    }

    impl<T> WithNotion<State<T>> for State<T>
    where
        T: PartialEq + Default + 'static,
    {
        fn apply(self: Rc<Self>, notion: Rc<State<T>>) -> Rc<Self> {
            notion
        }
    }

    #[function_component(Comp)]
    fn comp() -> Html {
        let a = use_atom::<State<u32>>();
        let b = use_atom::<State<u64>>();

        {
            let a = a.clone();
            let b = b.clone();
            use_effect_with((), move |_| {
                a.set(State { inner: 1 });
                b.set(State { inner: 2 });

                || {}
            });
        }

        html! {
            <div>
                <div id="a">{a.inner}</div>
                <div id="b">{b.inner}</div>
            </div>
        }
    }

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
    assert_eq!(s, "1");

    let s = get_text_content("#b").await;
    assert_eq!(s, "2");
}
