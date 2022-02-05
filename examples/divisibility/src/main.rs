use std::rc::Rc;

use bounce::prelude::*;
use bounce::BounceRoot;
use log::Level;
use stylist::yew::styled_component;
use yew::prelude::*;

#[derive(Debug)]
pub enum SliceAction {
    Increment,
}

#[derive(Default, PartialEq, Slice)]
pub struct Value(i64);

impl Reducible for Value {
    type Action = SliceAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Increment => Self(self.0 + 1).into(),
        }
    }
}

#[derive(PartialEq)]
pub struct IsEven {
    inner: bool,
}

impl Selector for IsEven {
    fn select(states: &BounceStates) -> Rc<Self> {
        let val = states.get_slice_value::<Value>();

        Self {
            inner: val.0 % 2 == 0,
        }
        .into()
    }
}

#[derive(PartialEq)]
pub struct Div3 {
    inner: bool,
}

impl Selector for Div3 {
    fn select(states: &BounceStates) -> Rc<Self> {
        let val = states.get_slice_value::<Value>();

        Self {
            inner: val.0 % 3 == 0,
        }
        .into()
    }
}

#[derive(PartialEq)]
pub struct Div4 {
    inner: bool,
}

impl Selector for Div4 {
    fn select(states: &BounceStates) -> Rc<Self> {
        let val = states.get_slice_value::<Value>();

        Self {
            inner: val.0 % 4 == 0,
        }
        .into()
    }
}

#[styled_component(CompIsEven)]
fn comp_is_even() -> Html {
    let value = use_slice_value::<Value>();
    let is_even = use_selector_value::<IsEven>();

    let maybe_not = if is_even.inner { "" } else { " not" };

    html! {
        <div>
            <p>{format!("{} is{} even.", value.0, maybe_not)}</p>
        </div>
    }
}

#[styled_component(CompDiv3)]
fn comp_div_3() -> Html {
    let value = use_slice_value::<Value>();
    let is_div_3 = use_selector_value::<Div3>();

    let maybe_not = if is_div_3.inner { "" } else { " not" };

    html! {
        <div>
            <p>{format!("{} is{} divisible by 3.", value.0, maybe_not)}</p>
        </div>
    }
}

#[styled_component(CompDiv4)]
fn comp_div_4() -> Html {
    let value = use_slice_value::<Value>();
    let is_div_4 = use_selector_value::<Div4>();

    let maybe_not = if is_div_4.inner { "" } else { " not" };

    html! {
        <div>
            <p>{format!("{} is{} divisible by 4.", value.0, maybe_not)}</p>
        </div>
    }
}

#[styled_component(Setters)]
fn setters() -> Html {
    let dispatch = use_slice_dispatch::<Value>();

    let inc = Callback::from(move |_| dispatch(SliceAction::Increment));

    html! {
        <div>
            <button onclick={inc}>{"Increase Value"}</button>
        </div>
    }
}

#[styled_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <div>
                <div class={css!(r#"
                    grid-template-columns: auto auto auto;
                    display: grid;

                    width: 600px;
                "#)}>
                    <CompIsEven />
                    <CompDiv3 />
                    <CompDiv4 />
                </div>
                <Setters />
            </div>
        </BounceRoot>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::start_app::<App>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use gloo::utils::document;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_divisibility() {
        yew::start_app_in_element::<App>(document().query_selector("#output").unwrap().unwrap());
    }
}
