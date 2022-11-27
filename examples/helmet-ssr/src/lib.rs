use bounce::helmet::{Helmet, HelmetBridge};
use bounce::BounceRoot;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(PartialEq, Eq)]
pub struct Title {
    value: String,
}

#[derive(Routable, PartialEq, Clone, Eq)]
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

fn format_title(s: AttrValue) -> AttrValue {
    format!("{} - Example", s).into()
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BounceRoot>
            <HelmetBridge default_title="Example" {format_title} />
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

#[cfg(feature = "ssr")]
mod feat_ssr {
    use super::*;

    use std::collections::HashMap;

    use bounce::helmet::StaticWriter;
    use yew_router::history::{AnyHistory, History, MemoryHistory};

    #[derive(Properties, PartialEq, Eq, Debug)]

    pub struct ServerAppProps {
        pub url: AttrValue,
        pub queries: HashMap<String, String>,
        pub helmet_writer: StaticWriter,
    }

    #[function_component]
    pub fn ServerApp(props: &ServerAppProps) -> Html {
        let history = AnyHistory::from(MemoryHistory::new());
        history
            .push_with_query(&*props.url, &props.queries)
            .unwrap();

        html! {
            <BounceRoot>
                <HelmetBridge
                    default_title="Example"
                    {format_title}
                    writer={props.helmet_writer.clone()}
                />
                <Helmet>
                    <meta charset="utf-8" />
                    <meta name="description" content="default page" />
                </Helmet>
                <Router {history}>
                    <Switch<Route> render={render_fn} />
                    <Links />
                </Router>
            </BounceRoot>
        }
    }
}

#[cfg(feature = "ssr")]
pub use feat_ssr::*;
