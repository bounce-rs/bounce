use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

use web_sys::Element;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

use super::state::{HelmetState, HelmetTag};
use crate::states::artifact::use_artifacts;

/// Properties of the [HelmetBridge]
#[derive(Properties, Clone)]
pub struct HelmetBridgeProps {
    /// The default title to apply if no title is provided.
    #[prop_or_default]
    pub default_title: Option<AttrValue>,

    /// The function to format title.
    #[prop_or_default]
    pub format_title: Option<Rc<dyn Fn(&str) -> String>>,
}

impl fmt::Debug for HelmetBridgeProps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelmetBridgeProps")
            .field("default_title", &self.default_title)
            .field(
                "format_title",
                if self.format_title.is_some() {
                    &"Some(_)"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl PartialEq for HelmetBridgeProps {
    fn eq(&self, rhs: &Self) -> bool {
        let format_title_eq = match (&self.format_title, &rhs.format_title) {
            (Some(ref m), Some(ref n)) => Rc::ptr_eq(m, n),
            (None, None) => true,
            _ => false,
        };

        format_title_eq && self.default_title == rhs.default_title
    }
}

/// The Helmet Bridge.
///
/// This component is responsible to reconclie all helmet tags to the real dom.
///
/// It accepts two props, a string `default_title` and a function `format_title`.
///
/// You can only register 1 `HelmetBridge` per `BounceRoot`. Having multiple `HelmetBridge`s
/// under the same bounce root may cause unexpected results.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # use bounce::helmet::HelmetBridge;
///
/// # #[function_component(Comp)]
/// # fn comp() -> Html {
///     html! {
///         <BounceRoot>
///             <HelmetBridge default_title="default title" />
///             // other components.
///         </BounceRoot>
///     }
/// # }
/// ```
#[function_component(HelmetBridge)]
pub fn helmet_bridge(props: &HelmetBridgeProps) -> Html {
    let helmet_states = use_artifacts::<HelmetState>();

    let rendered = use_mut_ref(|| -> Option<BTreeMap<Rc<HelmetTag>, Option<Element>>> { None });

    use_effect_with_deps(
        move |(helmet_states, props)| {
            // Calculate tags to render.
            let mut title: Option<Rc<str>> = None;

            for state in helmet_states {
                for tag in state.tags.iter() {
                    match **tag {
                        HelmetTag::Title(ref m) => {
                            title = Some(m.clone());
                        }
                    }
                }
            }

            let mut to_render = BTreeSet::new();

            // calculate title from it.
            if let Some(m) = title
                .map(|m| {
                    props
                        .format_title
                        .as_ref()
                        .map(|fmt_fn| Rc::<str>::from(fmt_fn(&m)))
                        .unwrap_or(m)
                })
                .or_else(|| props.default_title.as_ref().map(|m| m.to_string().into()))
            {
                to_render.insert(HelmetTag::Title(m));
            }

            // Render tags with consideration of currently rendered tags.
            let mut rendered = rendered.borrow_mut();
            let mut last_rendered = rendered.take();

            let mut current_rendered = BTreeMap::new();

            let mut next_last_rendered = None;
            for next_to_render in to_render.into_iter() {
                'inner: loop {
                    next_last_rendered = next_last_rendered.or_else(|| {
                        last_rendered.as_mut().and_then(|last_rendered| {
                            last_rendered
                                .keys()
                                .next()
                                .cloned()
                                .and_then(|m| last_rendered.remove_entry(&*m))
                        })
                    });

                    match &mut next_last_rendered {
                        Some((ref key, ref mut value)) => match (**key).cmp(&next_to_render) {
                            // next_last_rendered key is greater than next_to_render, render next_to_render
                            Ordering::Greater => {
                                let el = next_to_render.apply();

                                current_rendered.insert(Rc::new(next_to_render), el);

                                break 'inner;
                            }
                            // next_last_rendered key is less than next_to_render, remove next_last_rendered
                            Ordering::Less => {
                                key.detach(value.take());

                                next_last_rendered = None;
                            }
                            // next_last_rendered key is equal to next_to_render, move to
                            // current_rendered
                            Ordering::Equal => {
                                current_rendered.insert(Rc::new(next_to_render), value.take());

                                next_last_rendered = None;
                                break 'inner;
                            }
                        },
                        // We have reached the end of all previous render tags, we simply render
                        // next_to_render.
                        None => {
                            let el = next_to_render.apply();

                            current_rendered.insert(Rc::new(next_to_render), el);

                            break 'inner;
                        }
                    }
                }
            }

            *rendered = Some(current_rendered);

            || {}
        },
        (helmet_states, props.clone()),
    );

    Html::default()
}
