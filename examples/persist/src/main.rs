use std::fmt;
use std::rc::Rc;

use bounce::*;
use gloo::storage::{LocalStorage, Storage};
use log::Level;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::InputEvent;

#[derive(PartialEq, Atom)]
#[bounce(observed)]
struct Username {
    inner: String,
}

impl From<String> for Username {
    fn from(s: String) -> Self {
        Self { inner: s }
    }
}

impl Default for Username {
    fn default() -> Self {
        Self {
            inner: LocalStorage::get("username").unwrap_or_else(|_| "Jane Doe".into()),
        }
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Observed for Username {
    fn changed(self: Rc<Self>) {
        LocalStorage::set("username", &self.inner).expect("failed to set username.");
    }
}

#[function_component(Reader)]
fn reader() -> Html {
    let username = use_atom_value::<Username>();

    html! { <div id="reader">{"Hello, "}{username}</div> }
}

#[function_component(Setter)]
fn setter() -> Html {
    let username = use_atom::<Username>();

    let on_text_input = {
        let username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            username.set(input.value().into());
        })
    };

    html! {
        <div>
            <input id="input" type_="text" oninput={on_text_input} value={username.to_string()} />
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <Reader />
            <Setter />
            <div>{"Type a username, and it will be saved in the local storage."}</div>
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
    use web_sys::Event;
    use web_sys::EventTarget;
    use web_sys::HtmlInputElement;

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

    #[wasm_bindgen_test]
    async fn test_persist() {
        let handle =
            yew::Renderer::<App>::with_root(document().query_selector("#output").unwrap().unwrap())
                .render();

        assert_eq!(get_text_content_by_id("reader").await, "Hello, Jane Doe");

        document()
            .query_selector("#input")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .set_value("John Smith");

        document()
            .query_selector("#input")
            .unwrap()
            .unwrap()
            .unchecked_into::<EventTarget>()
            .dispatch_event(&Event::new("input").unwrap())
            .unwrap();

        assert_eq!(get_text_content_by_id("reader").await, "Hello, John Smith");

        handle.destroy();

        // make sure that app has been destroyed.
        assert_eq!(get_text_content_by_id("output").await, "");

        yew::Renderer::<App>::with_root(document().query_selector("#output").unwrap().unwrap())
            .render();

        assert_eq!(get_text_content_by_id("reader").await, "Hello, John Smith");
    }
}
