//! A module to manipulate common tags under the `<head />` element.
//!
//! The Helmet component supports the following elements:
//!
//! - `title`
//! - `style`
//! - `script`
//! - `html`
//! - `body`

mod bridge;
mod comp;
mod state;

pub use bridge::{HelmetBridge, HelmetBridgeProps};
pub use comp::{Helmet, HelmetProps};
