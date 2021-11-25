use std::fmt;

use bounce::*;
use log::Level;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::InputEvent;

#[derive(PartialEq, Atom)]
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
            inner: "Jane Doe".into(),
        }
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[function_component(Reader)]
fn reader() -> Html {
    let username = use_atom_value::<Username>();

    html! { <div>{"Hello, "}{username}</div> }
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
            <input type_="text" oninput={on_text_input} value={username.to_string()} />
        </div>
    }
}

#[function_component(Resetter)]
fn resetter() -> Html {
    let set_username = use_set_atom_value::<Username>();

    let on_reset_clicked = Callback::from(move |_| set_username(Username::default()));

    html! { <button onclick={on_reset_clicked}>{"Reset"}</button> }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <Reader />
            <Setter />
            <Resetter />
        </BounceRoot>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::start_app::<App>();
}
