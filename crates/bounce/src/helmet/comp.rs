use std::collections::BTreeMap;
use std::rc::Rc;

use wasm_bindgen::throw_str;
use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use super::state::{HelmetState, HelmetTag};
use crate::states::artifact::Artifact;

/// Properties for [Helmet].
#[derive(Properties, Debug, PartialEq)]
pub struct HelmetProps {
    /// Children of helmet tags.
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
    }
}

fn collect_text_content(tag: &VTag) -> String {
    collect_str_in_children(&tag.children().clone().into())
}

fn collect_attributes(tag: &VTag) -> BTreeMap<&'static str, Rc<str>> {
    tag.attributes
        .iter()
        .map(|(k, v)| (k, v.into()))
        .collect::<BTreeMap<&'static str, Rc<str>>>()
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
    }
}
fn assert_empty_children(tag: &VTag) {
    assert_empty_node(&tag.children().clone().into())
}

/// A component to register helmet tags.
#[function_component(Helmet)]
pub fn helmet(props: &HelmetProps) -> Html {
    let tags = props
        .children
        .clone()
        .into_iter()
        .map(|m| match m {
            VNode::VTag(m) => match m.tag() {
                "title" => HelmetTag::Title(collect_text_content(&m).into()).into(),

                "script" => {
                    let attrs = collect_attributes(&m);
                    let content: Rc<str> = collect_text_content(&m).into();

                    HelmetTag::Script { attrs, content }.into()
                }
                "style" => {
                    let attrs = collect_attributes(&m);
                    let content: Rc<str> = collect_text_content(&m).into();

                    HelmetTag::Style { attrs, content }.into()
                }

                "html" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    HelmetTag::Html { attrs }.into()
                }
                "body" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    HelmetTag::Body { attrs }.into()
                }

                "base" => {
                    assert_empty_children(&m);
                    let attrs = collect_attributes(&m);

                    HelmetTag::Base { attrs }.into()
                }
                _ => throw_str(&format!("unsupported helmet tag type: {}", m.tag())),
            },
            _ => throw_str(
                "unsupported helmet tag type, expect html tag of title, meta, link, script, style",
            ),
        })
        .collect::<Vec<Rc<HelmetTag>>>();

    let state = Rc::new(HelmetState { tags });

    html! {<Artifact<HelmetState> value={state} />}
}
