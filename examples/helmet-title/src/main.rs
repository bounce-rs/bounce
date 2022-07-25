use std::rc::Rc;

use bounce::helmet::{Helmet, HelmetBridge};
use bounce::BounceRoot;
use log::Level;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(PartialEq)]
pub struct Title {
    value: String,
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
            <Helmet>
                <title>{"Page A"}</title>
                <meta name="description" content="page A" />
                <style>{"html { font-family: sans-serif; }"}</style>
            </Helmet>
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
            <Helmet>
                <title>{"Home Page"}</title>
                <meta name="description" content="home page" />
            </Helmet>
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

fn render_fn(route: Route) -> Html {
    match route {
        Route::A => html! {<A />},
        Route::B => html! {<B />},
        Route::Home => html! {<Home />},
    }
}

fn format_title(s: &str) -> String {
    format!("{} - Example", s)
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <HelmetBridge default_title="Example" format_title={Some(Rc::new(format_title) as Rc<dyn Fn(&str) -> String>)} />
            <Helmet>
                <meta charset="utf-8" />
                <meta name="description" content="default page" />
            </Helmet>
            <BrowserRouter>
                <Switch<Route> render={render_fn} />
                <Links />
            </BrowserRouter>
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
    use web_sys::{HtmlElement, HtmlMetaElement};

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
        yew::Renderer::<App>::with_root(document().query_selector("#output").unwrap().unwrap())
            .render();

        sleep(Duration::ZERO).await;

        assert_eq!(document().title(), "Home Page - Example");

        sleep(Duration::ZERO).await;

        let description = document()
            .query_selector("meta[name='description']")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlMetaElement>()
            .content();
        assert_eq!("home page", description);

        click_by_id("go-to-a").await;

        sleep(Duration::ZERO).await;

        assert_eq!(document().title(), "Page A - Example");

        sleep(Duration::ZERO).await;

        let description = document()
            .query_selector("meta[name='description']")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlMetaElement>()
            .content();
        assert_eq!("page A", description);

        click_by_id("go-to-b").await;

        sleep(Duration::ZERO).await;

        assert_eq!(document().title(), "Example");

        sleep(Duration::ZERO).await;

        let description = document()
            .query_selector("meta[name='description']")
            .unwrap()
            .unwrap()
            .unchecked_into::<HtmlMetaElement>()
            .content();
        assert_eq!("default page", description);
    }
}
