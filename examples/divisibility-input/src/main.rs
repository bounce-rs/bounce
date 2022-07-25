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
pub struct DivBy {
    inner: bool,
}

impl InputSelector for DivBy {
    type Input = i64;

    fn select(states: &BounceStates, input: Rc<i64>) -> Rc<Self> {
        let val = states.get_slice_value::<Value>();

        Self {
            inner: val.0 % *input == 0,
        }
        .into()
    }
}

#[styled_component(CompIsEven)]
fn comp_is_even() -> Html {
    let value = use_slice_value::<Value>();
    let is_even = use_input_selector_value::<DivBy>(2.into());

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
    let is_div_3 = use_input_selector_value::<DivBy>(3.into());

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
    let is_div_4 = use_input_selector_value::<DivBy>(4.into());

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
            <button onclick={inc} id="inc-btn">{"Increase Value"}</button>
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
    use web_sys::HtmlElement;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_divisibility_input() {
        yew::Renderer::<App>::with_root(document().query_selector("#output").unwrap().unwrap())
            .render();

        sleep(Duration::ZERO).await;

        let output = document()
            .query_selector("#output")
            .unwrap()
            .unwrap()
            .inner_html();

        assert!(output.contains("0 is even"));
        assert!(output.contains("0 is divisible by 3"));
        assert!(output.contains("0 is divisible by 4"));

        document()
            .query_selector("#inc-btn")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlElement>()
            .click();

        sleep(Duration::ZERO).await;

        let output = document()
            .query_selector("#output")
            .unwrap()
            .unwrap()
            .inner_html();

        assert!(output.contains("1 is not even"));
        assert!(output.contains("1 is not divisible by 3"));
        assert!(output.contains("1 is not divisible by 4"));

        document()
            .query_selector("#inc-btn")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlElement>()
            .click();

        sleep(Duration::ZERO).await;

        let output = document()
            .query_selector("#output")
            .unwrap()
            .unwrap()
            .inner_html();

        assert!(output.contains("2 is even"));
        assert!(output.contains("2 is not divisible by 3"));
        assert!(output.contains("2 is not divisible by 4"));

        document()
            .query_selector("#inc-btn")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlElement>()
            .click();

        sleep(Duration::ZERO).await;

        let output = document()
            .query_selector("#output")
            .unwrap()
            .unwrap()
            .inner_html();

        assert!(output.contains("3 is not even"));
        assert!(output.contains("3 is divisible by 3"));
        assert!(output.contains("3 is not divisible by 4"));

        document()
            .query_selector("#inc-btn")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlElement>()
            .click();

        sleep(Duration::ZERO).await;

        let output = document()
            .query_selector("#output")
            .unwrap()
            .unwrap()
            .inner_html();

        assert!(output.contains("4 is even"));
        assert!(output.contains("4 is not divisible by 3"));
        assert!(output.contains("4 is divisible by 4"));
    }
}
