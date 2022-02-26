use std::collections::BTreeMap;
use std::rc::Rc;

use gloo::utils::document;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlHeadElement, HtmlScriptElement, HtmlStyleElement};

thread_local! {
    static HEAD: HtmlHeadElement = document().head().unwrap_throw();
    static HTML_TAG: Element = document().document_element().unwrap_throw();
    static BODY_TAG: HtmlElement = document().body().unwrap_throw();
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
    Style {
        content: Rc<str>,
        attrs: BTreeMap<&'static str, Rc<str>>,
    },
    Html {
        attrs: BTreeMap<&'static str, Rc<str>>,
    },
    Body {
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

pub(crate) fn add_class_list(element: &Element, classes_str: &str) {
    let class_list = element.class_list();

    for class in classes_str.split_whitespace() {
        class_list.add_1(class).expect_throw("failed to add class");
    }
}

pub(crate) fn remove_class_list(element: &Element, classes_str: &str) {
    let class_list = element.class_list();

    for class in classes_str.split_whitespace() {
        class_list
            .remove_1(class)
            .expect_throw("failed to remove class");
    }
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
                    match *name {
                        "type" => {
                            el.set_type(value);
                        }
                        "class" => {
                            add_class_list(&el, &*value);
                        }
                        _ => {
                            el.set_attribute(name, value)
                                .expect_throw("failed to set script attribute");
                        }
                    }
                }

                append_to_head(&el);

                Some(el.into())
            }

            Self::Style { content, attrs } => {
                let el = create_element::<HtmlStyleElement>("style");

                el.append_child(&document().create_text_node(content))
                    .expect_throw("failed to set style content");

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            add_class_list(&el, &*value);
                        }
                        _ => {
                            el.set_attribute(name, value)
                                .expect_throw("failed to set style attribute");
                        }
                    }
                }

                append_to_head(&el);

                Some(el.into())
            }

            Self::Html { attrs } => {
                let el = HTML_TAG.with(|m| m.clone());

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            add_class_list(&el, &*value);
                        }
                        _ => {
                            el.set_attribute(name, value)
                                .expect_throw("failed to set html attribute");
                        }
                    }
                }

                None
            }

            Self::Body { attrs } => {
                let el = BODY_TAG.with(|m| m.clone());

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            add_class_list(&el, &*value);
                        }
                        _ => {
                            el.set_attribute(name, value)
                                .expect_throw("failed to set body attribute");
                        }
                    }
                }

                None
            }
        }
    }

    pub fn detach(&self, element: Option<Element>) {
        if let Some(m) = element {
            m.parent_element().as_ref().map(|m| m.remove_child(m));
        }

        match self {
            Self::Title(_) => {}
            Self::Script { .. } => {}
            Self::Style { .. } => {}
            Self::Html { attrs } => {
                let el = HTML_TAG.with(|m| m.clone());

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            remove_class_list(&el, &*value);
                        }
                        _ => {
                            el.remove_attribute(name)
                                .expect_throw("failed to remove html attribute");
                        }
                    }
                }
            }
            Self::Body { attrs } => {
                let el = BODY_TAG.with(|m| m.clone());

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            remove_class_list(&el, &*value);
                        }
                        _ => {
                            el.remove_attribute(name)
                                .expect_throw("failed to remove html attribute");
                        }
                    }
                }
            }
        }
    }
}
