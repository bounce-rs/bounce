use log::Level;
use queries_ssr::App;

fn main() {
    console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::Renderer::<App>::new().hydrate();
}
