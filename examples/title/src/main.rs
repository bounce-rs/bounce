//! An example to demonstrate the artifact API.
//!
//! In your application, if you want to change title based on the content rendered,
//! you may want to use the helmet API instead. It's a high-level API that's built with the
//! Artifact API.

use std::rc::Rc;

use bounce::prelude::*;
use bounce::BounceRoot;
use gloo::utils::document;
use log::Level;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(PartialEq)]
pub struct Title {
    value: String,
}

/// A component to apply the title when it changes.
#[function_component(TitleApplier)]
pub fn title_applier() -> Html {
    let titles = use_artifacts::<Title>();

    let title = titles
        .last()
        .map(|m| m.value.to_owned())
        .unwrap_or_else(|| "unknown title".into());

    use_effect_with_deps(
        |m| {
            document().set_title(m);

            || {}
        },
        title,
    );

    Html::default()
}

#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    #[at("/a")]
    A,
    #[at("/b")]
    B,
    #[at("/")]
    Home,
}

#[function_component(A)]
fn a() -> Html {
    html! {
        <>
            <Artifact<Title> value={Rc::new(Title { value: "Page A - Example".into() })} />
            <div>{"This is page A."}</div>
        </>
    }
}

#[function_component(B)]
fn b() -> Html {
    html! {
        <div>{"This is page B. This page does not have a specific title, so default title will be used instead."}</div>
    }
}

#[function_component(Home)]
fn home() -> Html {
    html! {
        <>
            <Artifact<Title> value={Rc::new(Title { value: "Home Page - Example".into() })} />
            <div>{"This is home page."}</div>
        </>
    }
}

#[function_component(Links)]
fn links() -> Html {
    html! {
        <>
            <div><Link<Route> to={Route::A}><span id="go-to-a">{"Go to A"}</span></Link<Route>></div>
            <div><Link<Route> to={Route::B}><span id="go-to-b">{"Go to B"}</span></Link<Route>></div>
            <div><Link<Route> to={Route::Home}><span id="go-to-home">{"Go to Home"}</span></Link<Route>></div>
        </>
    }
}

fn render_fn(route: &Route) -> Html {
    match route {
        Route::A => html! {<A />},
        Route::B => html! {<B />},
        Route::Home => html! {<Home />},
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <TitleApplier />
            <Artifact<Title> value={Rc::new(Title { value: "Example".into() })} />
            <BrowserRouter>
                <Switch<Route> render={Switch::render(render_fn)} />
                <Links />
            </BrowserRouter>
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
    async fn test_title() {
        yew::start_app_in_element::<App>(document().query_selector("#output").unwrap().unwrap());

        sleep(Duration::ZERO).await;

        assert_eq!(document().title(), "Home Page - Example");

        click_by_id("go-to-a").await;

        sleep(Duration::ZERO).await;

        assert_eq!(document().title(), "Page A - Example");

        click_by_id("go-to-b").await;

        sleep(Duration::ZERO).await;

        assert_eq!(document().title(), "Example");
    }
}
