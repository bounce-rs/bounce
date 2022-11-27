use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use wasm_bindgen::throw_str;
use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use super::state::{HelmetState, HelmetTag};
use crate::states::artifact::Artifact;
use crate::utils::Id;

/// Properties for [Helmet].
#[derive(Properties, Debug, PartialEq)]
pub struct HelmetProps {
    /// Children of the [Helmet] component.
    ///
    /// This property only accepts a list of elements denoted in the module documentation.
    #[prop_or_default]
    pub children: Children,
}

fn collect_str_in_children(tag: &VNode) -> String {
    match tag {
        VNode::VTag(_) => throw_str("expected text content, found tag."),
        VNode::VText(ref m) => m.text.to_string(),
        VNode::VList(ref m) => {
            let mut s = "".to_string();

            for i in m.iter() {
                s.push_str(&collect_str_in_children(i));
            }

            s
        }
        VNode::VComp(_) => throw_str("expected text content, found component."),
        VNode::VPortal(_) => throw_str("expected text content, found portal."),
        VNode::VRef(_) => throw_str("expected text content, found node reference."),
        VNode::VSuspense(_) => throw_str("expected text content, found suspense."),
        VNode::VRaw(_) => throw_str("expected text content, found raw html."),
    }
}

fn collect_text_content(tag: &VTag) -> String {
    collect_str_in_children(&tag.children().clone().into())
}

fn collect_attributes(tag: &VTag) -> BTreeMap<Arc<str>, Arc<str>> {
    let mut map = BTreeMap::new();

    for (k, v) in tag.attributes.iter() {
        map.insert(k.into(), v.into());
    }

    map
}

fn assert_empty_node(node: &VNode) {
    match node {
        VNode::VTag(_) => throw_str("expected nothing, found tag."),
        VNode::VText(_) => throw_str("expected nothing, found text content."),
        VNode::VList(ref m) => {
            for node in m.iter() {
                assert_empty_node(node);
            }
        }
        VNode::VComp(_) => throw_str("expected nothing, found component."),
        VNode::VPortal(_) => throw_str("expected nothing, found portal."),
        VNode::VRef(_) => throw_str("expected nothing, found node reference."),
        VNode::VSuspense(_) => throw_str("expected nothing, found suspense."),
        VNode::VRaw(_) => throw_str("expected nothing, found raw html."),
    }
}
fn assert_empty_children(tag: &VTag) {
    assert_empty_node(&tag.children().clone().into())
}

#[derive(Properties, PartialEq, Clone)]
struct ScriptHelmetProps {
    attrs: BTreeMap<Arc<str>, Arc<str>>,
    content: Arc<str>,
}

// A special component to render the script tag with a unique id.
#[function_component(ScriptHelmet)]
fn script_helmet(props: &ScriptHelmetProps) -> Html {
    let id = *use_state(Id::new);
    let ScriptHelmetProps { attrs, content } = props.clone();

    let tags = vec![Arc::new(HelmetTag::Script {
        attrs,
        content,
        _id: id,
    })];
    let state = Rc::new(HelmetState { tags });

    html! {<Artifact<HelmetState> value={state} />}
}

/// A component to register head elements.
///
/// # Panics
///
/// This component will panic if unsupported elements are passed as children.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use bounce::BounceRoot;
/// # use bounce::prelude::*;
/// use bounce::helmet::Helmet;
///
/// # #[function_component(Comp)]
/// # fn comp() -> Html {
/// html! {
///     <Helmet>
///         // The title to apply.
///         <title>{"page a title"}</title>
///     </Helmet>
/// }
/// # }
/// ```
#[function_component(Helmet)]
pub fn helmet(props: &HelmetProps) -> Html {
    let mut script_helmets = Vec::new();

    let tags = props
        .children
        .clone()
        .into_iter()
        .filter_map(|m| match m {
            VNode::VTag(m) => match m.tag() {
                "title" => Some(HelmetTag::Title(collect_text_content(&m).into()).into()),

                "script" => {
                    let attrs = collect_attributes(&m);
                    let content: Arc<str> = collect_text_content(&m).into();

                    script_helmets.push(html! { <ScriptHelmet {attrs} {content} /> });

                    None
                }
                "style" => {
                    let attrs = collect_attributes(&m);
                    let content: Arc<str> = collect_text_content(&m).into();

                    Some(HelmetTag::Style { attrs, content }.into())
                }

                "html" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    Some(HelmetTag::Html { attrs }.into())
                }
                "body" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    Some(HelmetTag::Body { attrs }.into())
                }

                "base" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    Some(HelmetTag::Base { attrs }.into())
                }
                "link" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    Some(HelmetTag::Link { attrs }.into())
                }
                "meta" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    Some(HelmetTag::Meta { attrs }.into())
                }
                _ => throw_str(&format!("unsupported helmet tag type: {}", m.tag())),
            },
            _ => throw_str("unsupported helmet node type, expect a supported helmet tag."),
        })
        .collect::<Vec<Arc<HelmetTag>>>();

    let state = Rc::new(HelmetState { tags });

    html! {
        <>
            <Artifact<HelmetState> value={state} />
            {script_helmets}
        </>
    }
}
