use std::rc::Rc;

use wasm_bindgen::throw_str;
use yew::html::ChildrenRenderer;
use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use super::state::{HelmetState, HelmetTag};
use crate::states::artifact::Artifact;

#[derive(Properties, Debug, PartialEq)]
pub struct HelmetProps {
    #[prop_or_default]
    children: ChildrenRenderer<VTag>,
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

#[function_component(Helmet)]
pub fn helmet(props: &HelmetProps) -> Html {
    let tags = props
        .children
        .clone()
        .into_iter()
        .map(|m| match m.tag() {
            "title" => HelmetTag::Title(collect_text_content(&m)).into(),
            _ => throw_str(&format!("unsupported helmet tag type: {}", m.tag())),
        })
        .collect::<Vec<Rc<HelmetTag>>>();

    let state = Rc::new(HelmetState { tags });

    html! {<Artifact<HelmetState> value={state} />}
}
