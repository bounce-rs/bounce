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
//! use bounce::helmet::{Helmet, HelmetProvider};
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
//!             // A helmet provider is required to apply helmet elements to the head element.
//!             // You only need 1 helmet provider per bounce root.
//!             // The helmet provider is intended to live as long as the BounceRoot.
//!             <HelmetProvider default_title="default title">
//!                 <Helmet>
//!                     // The title to apply.
//!                     //
//!                     // However, as <PageA /> also renders a title element, elements rendered later
//!                     // will have a higher priority. Hence, "page a title" will become the document
//!                     // title.
//!                     <title>{"app title"}</title>
//!                 </Helmet>
//!                 <PageA />
//!             </HelmetProvider>
//!         </BounceRoot>
//!     }
//! }
//! ```

use yew::prelude::*;

mod comp;
mod provider;
mod ssr;
mod state;

pub use comp::{Helmet, HelmetProps};
pub use provider::{HelmetProvider, HelmetProviderProps};
pub use ssr::{StaticRenderer, StaticWriter};

type FormatTitle = Callback<AttrValue, AttrValue>;
