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
pub struct SliceA(i64);

impl Reducible for SliceA {
    type Action = SliceAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Increment => Self(self.0 + 1).into(),
        }
    }
}

#[derive(Default, PartialEq, Slice)]
pub struct SliceB(i64);

impl Reducible for SliceB {
    type Action = SliceAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Increment => Self(self.0 + 1).into(),
        }
    }
}

#[derive(Default, PartialEq, Slice)]
pub struct SliceC(i64);

impl Reducible for SliceC {
    type Action = SliceAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Increment => Self(self.0 + 1).into(),
        }
    }
}

#[styled_component(CompA)]
fn comp_a() -> Html {
    let a = use_slice_value::<SliceA>();

    let ctr = {
        let ctr = use_mut_ref(|| 0);

        let mut ctr = ctr.borrow_mut();

        *ctr += 1;

        *ctr
    };

    html! {
        <div>
            <p id="val-a-a">{"Slice A: "}{a.0}</p>
            <p id="val-a-render-ctr">{format!("Rendered: {} Time(s)", ctr)}</p>
        </div>
    }
}

#[styled_component(CompB)]
fn comp_b() -> Html {
    let b = use_slice_value::<SliceB>();

    let ctr = {
        let ctr = use_mut_ref(|| 0);

        let mut ctr = ctr.borrow_mut();

        *ctr += 1;

        *ctr
    };

    html! {
        <div>
            <p id="val-b-b">{"Slice B: "}{b.0}</p>
            <p id="val-b-render-ctr">{format!("Rendered: {} Time(s)", ctr)}</p>
        </div>
    }
}

#[styled_component(CompC)]
fn comp_c() -> Html {
    let c = use_slice_value::<SliceC>();

    let ctr = {
        let ctr = use_mut_ref(|| 0);

        let mut ctr = ctr.borrow_mut();

        *ctr += 1;

        *ctr
    };

    html! {
        <div>
            <p id="val-c-c">{"Slice C: "}{c.0}</p>
            <p id="val-c-render-ctr">{format!("Rendered: {} Time(s)", ctr)}</p>
        </div>
    }
}

#[styled_component(CompAB)]
fn comp_ab() -> Html {
    let a = use_slice_value::<SliceA>();
    let b = use_slice_value::<SliceB>();

    let ctr = {
        let ctr = use_mut_ref(|| 0);

        let mut ctr = ctr.borrow_mut();

        *ctr += 1;

        *ctr
    };

    html! {
        <div>
            <p id="val-ab-a">{"Slice A: "}{a.0}</p>
            <p id="val-ab-b">{"Slice B: "}{b.0}</p>
            <p id="val-ab-render-ctr">{format!("Rendered: {} Time(s)", ctr)}</p>
        </div>
    }
}

#[styled_component(CompAC)]
fn comp_ac() -> Html {
    let a = use_slice_value::<SliceA>();
    let c = use_slice_value::<SliceC>();

    let ctr = {
        let ctr = use_mut_ref(|| 0);

        let mut ctr = ctr.borrow_mut();

        *ctr += 1;

        *ctr
    };

    html! {
        <div>
            <p id="val-ac-a">{"Slice A: "}{a.0}</p>
            <p id="val-ac-c">{"Slice C: "}{c.0}</p>
            <p id="val-ac-render-ctr">{format!("Rendered: {} Time(s)", ctr)}</p>
        </div>
    }
}

#[styled_component(CompBC)]
fn comp_bc() -> Html {
    let b = use_slice_value::<SliceB>();
    let c = use_slice_value::<SliceC>();

    let ctr = {
        let ctr = use_mut_ref(|| 0);

        let mut ctr = ctr.borrow_mut();

        *ctr += 1;

        *ctr
    };

    html! {
        <div>
            <p id="val-bc-b">{"Slice B: "}{b.0}</p>
            <p id="val-bc-c">{"Slice C: "}{c.0}</p>
            <p id="val-bc-render-ctr">{format!("Rendered: {} Time(s)", ctr)}</p>
        </div>
    }
}

#[styled_component(CompABC)]
fn comp_abc() -> Html {
    let a = use_slice_value::<SliceA>();
    let b = use_slice_value::<SliceB>();
    let c = use_slice_value::<SliceC>();

    let ctr = {
        let ctr = use_mut_ref(|| 0);

        let mut ctr = ctr.borrow_mut();

        *ctr += 1;

        *ctr
    };

    html! {
        <div class={css!(r#"
            grid-column-start: 1;
            grid-column-end: 4;
        "#)}>
            <p id="val-abc-a">{"Slice A: "}{a.0}</p>
            <p id="val-abc-b">{"Slice B: "}{b.0}</p>
            <p id="val-abc-c">{"Slice C: "}{c.0}</p>
            <p id="val-abc-render-ctr">{format!("Rendered: {} Time(s)", ctr)}</p>
        </div>
    }
}

#[styled_component(Setters)]
fn setters() -> Html {
    let dispatch_a = use_slice_dispatch::<SliceA>();
    let dispatch_b = use_slice_dispatch::<SliceB>();
    let dispatch_c = use_slice_dispatch::<SliceC>();

    let inc_a = Callback::from(move |_| dispatch_a(SliceAction::Increment));
    let inc_b = Callback::from(move |_| dispatch_b(SliceAction::Increment));
    let inc_c = Callback::from(move |_| dispatch_c(SliceAction::Increment));

    html! {
        <div class={css!(r#"
        "#)}>
            <button id="btn-inc-a" onclick={inc_a}>{"Increase A"}</button>
            <button id="btn-inc-b" onclick={inc_b}>{"Increase B"}</button>
            <button id="btn-inc-c" onclick={inc_c}>{"Increase C"}</button>
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
                    <CompA />
                    <CompB />
                    <CompC />

                    <CompAB />
                    <CompAC />
                    <CompBC />

                    <CompABC />
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
    use gloo::timers::future::sleep;
    use gloo::utils::document;
    use std::time::Duration;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use web_sys::HtmlElement;

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
    async fn test_partial_render() {
        yew::start_app_in_element::<App>(document().query_selector("#output").unwrap().unwrap());

        assert_eq!(get_text_content_by_id("val-a-a").await, "Slice A: 0");
        assert_eq!(get_text_content_by_id("val-ab-a").await, "Slice A: 0");
        assert_eq!(get_text_content_by_id("val-ac-a").await, "Slice A: 0");
        assert_eq!(get_text_content_by_id("val-abc-a").await, "Slice A: 0");

        assert_eq!(get_text_content_by_id("val-b-b").await, "Slice B: 0");
        assert_eq!(get_text_content_by_id("val-ab-b").await, "Slice B: 0");
        assert_eq!(get_text_content_by_id("val-bc-b").await, "Slice B: 0");
        assert_eq!(get_text_content_by_id("val-abc-b").await, "Slice B: 0");

        assert_eq!(get_text_content_by_id("val-c-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-ac-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-bc-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-abc-c").await, "Slice C: 0");

        assert_eq!(
            get_text_content_by_id("val-a-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-b-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-c-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ab-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ac-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-bc-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-abc-render-ctr").await,
            "Rendered: 1 Time(s)"
        );

        click_by_id("btn-inc-a").await;

        assert_eq!(get_text_content_by_id("val-a-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-ab-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-ac-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-abc-a").await, "Slice A: 1");

        assert_eq!(get_text_content_by_id("val-b-b").await, "Slice B: 0");
        assert_eq!(get_text_content_by_id("val-ab-b").await, "Slice B: 0");
        assert_eq!(get_text_content_by_id("val-bc-b").await, "Slice B: 0");
        assert_eq!(get_text_content_by_id("val-abc-b").await, "Slice B: 0");

        assert_eq!(get_text_content_by_id("val-c-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-ac-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-bc-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-abc-c").await, "Slice C: 0");

        assert_eq!(
            get_text_content_by_id("val-a-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-b-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-c-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ab-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ac-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-bc-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-abc-render-ctr").await,
            "Rendered: 2 Time(s)"
        );

        click_by_id("btn-inc-b").await;

        assert_eq!(get_text_content_by_id("val-a-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-ab-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-ac-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-abc-a").await, "Slice A: 1");

        assert_eq!(get_text_content_by_id("val-b-b").await, "Slice B: 1");
        assert_eq!(get_text_content_by_id("val-ab-b").await, "Slice B: 1");
        assert_eq!(get_text_content_by_id("val-bc-b").await, "Slice B: 1");
        assert_eq!(get_text_content_by_id("val-abc-b").await, "Slice B: 1");

        assert_eq!(get_text_content_by_id("val-c-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-ac-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-bc-c").await, "Slice C: 0");
        assert_eq!(get_text_content_by_id("val-abc-c").await, "Slice C: 0");

        assert_eq!(
            get_text_content_by_id("val-a-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-b-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-c-render-ctr").await,
            "Rendered: 1 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ab-render-ctr").await,
            "Rendered: 3 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ac-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-bc-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-abc-render-ctr").await,
            "Rendered: 3 Time(s)"
        );

        click_by_id("btn-inc-c").await;

        assert_eq!(get_text_content_by_id("val-a-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-ab-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-ac-a").await, "Slice A: 1");
        assert_eq!(get_text_content_by_id("val-abc-a").await, "Slice A: 1");

        assert_eq!(get_text_content_by_id("val-b-b").await, "Slice B: 1");
        assert_eq!(get_text_content_by_id("val-ab-b").await, "Slice B: 1");
        assert_eq!(get_text_content_by_id("val-bc-b").await, "Slice B: 1");
        assert_eq!(get_text_content_by_id("val-abc-b").await, "Slice B: 1");

        assert_eq!(get_text_content_by_id("val-c-c").await, "Slice C: 1");
        assert_eq!(get_text_content_by_id("val-ac-c").await, "Slice C: 1");
        assert_eq!(get_text_content_by_id("val-bc-c").await, "Slice C: 1");
        assert_eq!(get_text_content_by_id("val-abc-c").await, "Slice C: 1");

        assert_eq!(
            get_text_content_by_id("val-a-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-b-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-c-render-ctr").await,
            "Rendered: 2 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ab-render-ctr").await,
            "Rendered: 3 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-ac-render-ctr").await,
            "Rendered: 3 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-bc-render-ctr").await,
            "Rendered: 3 Time(s)"
        );
        assert_eq!(
            get_text_content_by_id("val-abc-render-ctr").await,
            "Rendered: 4 Time(s)"
        );
    }
}
