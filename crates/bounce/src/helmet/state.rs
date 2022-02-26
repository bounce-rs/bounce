use std::collections::BTreeMap;
use std::rc::Rc;

use gloo::utils::document;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlHeadElement, HtmlScriptElement};

thread_local! {
    static HEAD: HtmlHeadElement = document().head().unwrap_throw();
}

#[derive(PartialEq, Debug)]
pub(crate) struct HelmetState {
    pub tags: Vec<Rc<HelmetTag>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum HelmetTag {
    Title(Rc<str>),
    Script {
        content: Rc<str>,
        attrs: BTreeMap<&'static str, Rc<str>>,
    },
}

pub(crate) fn create_element<T>(tag_name: &str) -> T
where
    T: AsRef<Element> + JsCast,
{
    let element = document().create_element(tag_name).unwrap_throw();

    JsValue::from(&element).dyn_into::<T>().unwrap_throw()
}

pub(crate) fn append_to_head(element: &Element) {
    HEAD.with(move |m| {
        m.append_child(element)
            .expect_throw("failed to append element to head.");
    })
}

impl HelmetTag {
    pub fn apply(&self) -> Option<Element> {
        match self {
            Self::Title(m) => {
                document().set_title(m);

                None
            }

            Self::Script { content, attrs } => {
                let el = create_element::<HtmlScriptElement>("script");

                el.set_text(content)
                    .expect_throw("failed to set script content");

                for (name, value) in attrs.iter() {
                    if name == &"type" {
                        el.set_type(value);
                    } else {
                        el.set_attribute(name, value)
                            .expect_throw("failed to set script attribute");
                    }
                }

                append_to_head(&el);

                Some(el.into())
            }
        }
    }

    pub fn detach(&self, element: Option<Element>) {
        if let Some(m) = element {
            m.parent_element().as_ref().map(|m| m.remove_child(m));
        }

        match self {
            Self::Title(_) => {}

            Self::Script { .. } => {
                todo!();
            }
        }
    }
}
