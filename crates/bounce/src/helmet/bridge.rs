use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::Element;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

use super::state::{HelmetState, HelmetTag};
use crate::root_state::BounceRootState;
use crate::states::artifact::use_artifacts;
use crate::states::slice::use_slice;
use crate::Slice;

enum HelmetBridgeGuardAction {
    Increment,
    Decrement,
}

/// A Guard to prevent multiple bridges to be registered.
#[derive(Default, PartialEq, Slice)]
struct HelmetBridgeGuard {
    inner: usize,
}

impl Reducible for HelmetBridgeGuard {
    type Action = HelmetBridgeGuardAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Increment => {
                debug_assert_eq!(
                    self.inner, 0,
                    "attempts to register more than 1 helmet bridge."
                );

                Self {
                    inner: self.inner + 1,
                }
                .into()
            }
            Self::Action::Decrement => Self {
                inner: self.inner - 1,
            }
            .into(),
        }
    }
}

/// Properties of the [HelmetBridge].
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

/// Applies attributes on top of existing attributes.
fn merge_attrs(
    target: &mut BTreeMap<&'static str, Rc<str>>,
    current_attrs: &BTreeMap<&'static str, Rc<str>>,
) {
    for (name, value) in current_attrs.iter() {
        match *name {
            "class" => match target.get(&"class").cloned() {
                Some(m) => {
                    target.insert(*name, Rc::<str>::from(format!("{} {}", value, m)));
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
}

/// Merges helmet states into a set of tags to be rendered.
fn merge_helmet_states(
    states: &[Rc<HelmetState>],
    props: &HelmetBridgeProps,
) -> BTreeSet<Rc<HelmetTag>> {
    let mut tags = BTreeSet::new();

    let mut title: Option<Rc<str>> = None;

    let mut html_attrs = BTreeMap::new();
    let mut body_attrs = BTreeMap::new();
    let mut base_attrs = BTreeMap::new();

    // BTreeMap<(rel, href), ..>
    let mut link_tags = BTreeMap::new();
    // BTreeMap<(name, http-equiv, scheme, charset), ..>
    let mut meta_tags = BTreeMap::new();

    for state in states {
        for tag in state.tags.iter() {
            match **tag {
                HelmetTag::Title(ref m) => {
                    title = Some(m.clone());
                }

                HelmetTag::Script { .. } => {
                    tags.insert(tag.clone());
                }

                HelmetTag::Style { .. } => {
                    tags.insert(tag.clone());
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
        tags.insert(HelmetTag::Title(m).into());
    }

    // html element.
    if !html_attrs.is_empty() {
        tags.insert(HelmetTag::Html { attrs: html_attrs }.into());
    }
    // body element.
    if !body_attrs.is_empty() {
        tags.insert(HelmetTag::Body { attrs: body_attrs }.into());
    }
    // base element.
    if !base_attrs.is_empty() {
        tags.insert(HelmetTag::Base { attrs: base_attrs }.into());
    }
    // link elements.
    tags.extend(link_tags.into_values());
    // meta elements.
    tags.extend(meta_tags.into_values());

    tags
}

/// Renders tags
fn render_tags(
    to_render: BTreeSet<Rc<HelmetTag>>,
    mut last_rendered: Option<BTreeMap<Rc<HelmetTag>, Option<Element>>>,
) -> BTreeMap<Rc<HelmetTag>, Option<Element>> {
    let mut rendered = BTreeMap::new();

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

                        rendered.insert(next_to_render, el);

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
                        rendered.insert(next_to_render, value.take());

                        next_last_rendered = None;
                        break 'inner;
                    }
                },
                // We have reached the end of all previous render tags, we simply render
                // next_to_render.
                None => {
                    let el = next_to_render.apply();

                    rendered.insert(next_to_render, el);

                    break 'inner;
                }
            }
        }
    }

    if let Some((key, value)) = next_last_rendered {
        key.detach(value);
    }

    if let Some(last_rendered) = last_rendered {
        for (key, value) in last_rendered.into_iter() {
            key.detach(value);
        }
    }

    rendered
}

/// The Helmet Bridge.
///
/// This component is responsible to reconclie all helmet tags to the real dom.
///
/// It accepts two properties, a `default_title` which will be applied when no other title elements
/// are registered and a `format_title` function which is used to format the title before it is
/// passed to the document.
///
/// # Panics
///
/// You can only register 1 `HelmetBridge` per `BounceRoot`. Registering multiple `HelmetBridge`s
/// will panic.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # use bounce::BounceRoot;
/// # use bounce::helmet::HelmetBridge;
/// #
/// # #[function_component(Comp)]
/// # fn comp() -> Html {
/// html! {
///     <BounceRoot>
///         <HelmetBridge default_title="default title" />
///         // other components.
///     </BounceRoot>
/// }
/// # }
/// ```
#[function_component(HelmetBridge)]
pub fn helmet_bridge(props: &HelmetBridgeProps) -> Html {
    let helmet_states = use_artifacts::<HelmetState>();
    let guard = use_slice::<HelmetBridgeGuard>();
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    let rendered = use_mut_ref(|| -> Option<BTreeMap<Rc<HelmetTag>, Option<Element>>> { None });

    use_effect_with_deps(
        move |_| {
            guard.dispatch(HelmetBridgeGuardAction::Increment);

            move || {
                guard.dispatch(HelmetBridgeGuardAction::Decrement);
            }
        },
        root,
    );

    use_effect_with_deps(
        move |(helmet_states, props)| {
            // Calculate tags to render.
            let to_render = merge_helmet_states(helmet_states, props);

            let mut rendered = rendered.borrow_mut();
            *rendered = Some(render_tags(to_render, rendered.take()));

            || {}
        },
        (helmet_states, props.clone()),
    );

    Html::default()
}
