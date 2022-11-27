//! A module to manipulate common tags under the `<head />` element.
//!
//! The Helmet component supports the following elements:
//!
//! - `title`
//! - `style`
//! - `script`
//! - `base`
//! - `link`
//! - `meta`
//!
//! The Helmet component supports setting attributes of the following elements:
//!
//! - `html`
//! - `body`
//!
//! # Example
//!
//! ```
//! # use yew::prelude::*;
//! # use bounce::BounceRoot;
//! # use bounce::prelude::*;
//! use bounce::helmet::{Helmet, HelmetBridge};
//!
//! #[function_component(PageA)]
//! fn page_a() -> Html {
//!     html! {
//!         <>
//!             <Helmet>
//!                 // The title to apply.
//!                 <title>{"page a title"}</title>
//!             </Helmet>
//!             <div>{"This is page A."}</div>
//!         </>
//!     }
//! }
//!
//! #[function_component(App)]
//! fn app() -> Html {
//!     html! {
//!         <BounceRoot>
//!             // A helmet bridge is required to apply helmet elements to the head element.
//!             // You only need 1 helmet bridge per bounce root.
//!             // The helmet bridge is intended to live as long as the BounceRoot.
//!             <HelmetBridge default_title="default title" />
//!             <Helmet>
//!                 // The title to apply.
//!                 //
//!                 // However, as <PageA /> also renders a title element, elements rendered later
//!                 // will have a higher priority. Hence, "page a title" will become the document
//!                 // title.
//!                 <title>{"app title"}</title>
//!             </Helmet>
//!             <PageA />
//!         </BounceRoot>
//!     }
//! }
//! ```
//!
//! Bounce Helmet also supports [Server-side rendering](render_static).

use yew::prelude::*;

mod bridge;
mod comp;
#[cfg(feature = "ssr")]
mod ssr;
mod state;

pub use bridge::{HelmetBridge, HelmetBridgeProps};
pub use comp::{Helmet, HelmetProps};
#[cfg(feature = "ssr")]
pub(crate) use ssr::StaticWriterState;
#[cfg(feature = "ssr")]
#[cfg_attr(documenting, doc(cfg(feature = "ssr")))]
pub use ssr::{render_static, StaticRenderer, StaticWriter};
pub use state::HelmetTag;

type FormatTitle = Callback<AttrValue, AttrValue>;
