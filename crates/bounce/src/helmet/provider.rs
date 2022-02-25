use std::rc::Rc;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

/// Properties of the [HelmetProvider]
#[derive(Properties)]
pub struct HelmetProviderProps {
    /// The default title to apply if no title is provided.
    #[prop_or_default]
    pub default_title: Option<AttrValue>,

    /// The function to format title.
    #[prop_or_default]
    pub format_title: Option<Rc<dyn Fn(&str) -> String>>,

    /// The children of the title provider.
    #[prop_or_default]
    pub children: Children,
}

#[allow(clippy::vtable_address_comparisons)]
impl PartialEq for HelmetProviderProps {
    fn eq(&self, rhs: &Self) -> bool {
        let format_title_eq = match (&self.format_title, &rhs.format_title) {
            (Some(ref m), Some(ref n)) => Rc::ptr_eq(m, n),
            (None, None) => true,
            _ => false,
        };

        format_title_eq && self.default_title == rhs.default_title && self.children == rhs.children
    }
}

/// The Helmet Provider.
///
/// This component is responsible to reconclie all helmet tags to the real dom.
///
/// It accepts two props, a string `default_title` and a function `format_title`.
///
/// You can only register 1 `HelmetProvider` per `BounceRoot`. Having multiple `HelmetProvider`s
/// under the same bounce root may cause unexpected results.
#[function_component(HelmetProvider)]
pub fn helmet_provider(props: &HelmetProviderProps) -> Html {
    let children = props.children.clone();

    html! {<>{children}</>}
}
