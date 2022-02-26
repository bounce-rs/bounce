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
/// # use bounce::BounceRoot;
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
            let mut to_render = BTreeSet::new();

            let mut title: Option<Rc<str>> = None;

            let mut html_attrs = BTreeMap::new();
            let mut body_attrs = BTreeMap::new();
            let mut base_attrs = BTreeMap::new();

            // BTreeMap<(rel, href), ..>
            let mut link_tags = BTreeMap::new();
            // BTreeMap<(name, http-equiv, scheme, charset), ..>
            let mut meta_tags = BTreeMap::new();

            let merge_attrs =
                |target: &mut BTreeMap<&'static str, Rc<str>>,
                 current_attrs: &BTreeMap<&'static str, Rc<str>>| {
                    for (name, value) in current_attrs.iter() {
                        match *name {
                            "class" => match target.get(&"class").cloned() {
                                Some(m) => {
                                    target
                                        .insert(*name, Rc::<str>::from(format!("{} {}", value, m)));
                                }
                                None => {
                                    target.insert(*name, value.clone());
                                }
                            },
                            _ => {
                                target.insert(*name, value.clone());
                            }
                        }
                    }
                };

            for state in helmet_states {
                for tag in state.tags.iter() {
                    match **tag {
                        HelmetTag::Title(ref m) => {
                            title = Some(m.clone());
                        }

                        HelmetTag::Script { .. } => {
                            to_render.insert(tag.clone());
                        }

                        HelmetTag::Style { .. } => {
                            to_render.insert(tag.clone());
                        }

                        HelmetTag::Html { ref attrs } => {
                            merge_attrs(&mut html_attrs, attrs);
                        }

                        HelmetTag::Body { ref attrs } => {
                            merge_attrs(&mut body_attrs, attrs);
                        }

                        HelmetTag::Base { ref attrs } => {
                            merge_attrs(&mut base_attrs, attrs);
                        }
                        HelmetTag::Link { ref attrs } => {
                            link_tags.insert(
                                (attrs.get(&"rel").cloned(), attrs.get(&"href").cloned()),
                                tag.clone(),
                            );
                        }
                        HelmetTag::Meta { ref attrs } => {
                            meta_tags.insert(
                                (
                                    attrs.get(&"name").cloned(),
                                    attrs.get(&"http-equiv").cloned(),
                                    attrs.get(&"scheme").cloned(),
                                    attrs.get(&"charset").cloned(),
                                ),
                                tag.clone(),
                            );
                        }
                    }
                }
            }

            // title.
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
                to_render.insert(HelmetTag::Title(m).into());
            }

            // html element.
            to_render.insert(HelmetTag::Html { attrs: html_attrs }.into());
            // body element.
            to_render.insert(HelmetTag::Body { attrs: body_attrs }.into());
            // base element.
            to_render.insert(HelmetTag::Base { attrs: base_attrs }.into());
            // link elements.
            to_render.extend(link_tags.into_values());
            // meta elements.
            to_render.extend(meta_tags.into_values());

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

                                current_rendered.insert(next_to_render, el);

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
                                current_rendered.insert(next_to_render, value.take());

                                next_last_rendered = None;
                                break 'inner;
                            }
                        },
                        // We have reached the end of all previous render tags, we simply render
                        // next_to_render.
                        None => {
                            let el = next_to_render.apply();

                            current_rendered.insert(next_to_render, el);

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
