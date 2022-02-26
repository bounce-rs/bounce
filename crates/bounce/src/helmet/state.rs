use std::collections::BTreeMap;
use std::rc::Rc;

use gloo::utils::document;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Element, HtmlBaseElement, HtmlElement, HtmlHeadElement, HtmlLinkElement, HtmlMetaElement,
    HtmlScriptElement, HtmlStyleElement,
};

use crate::utils::Id;

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
        // we need to always render script as long as they are not the same tag, so we use an Id to
        // distinguish between them.
        _id: Id,
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
    Base {
        attrs: BTreeMap<&'static str, Rc<str>>,
    },
    Link {
        attrs: BTreeMap<&'static str, Rc<str>>,
    },
    Meta {
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

            Self::Script { content, attrs, .. } => {
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

            Self::Base { attrs } => {
                let el = create_element::<HtmlBaseElement>("base");

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            add_class_list(&el, &*value);
                        }
                        _ => {
                            el.set_attribute(name, value)
                                .expect_throw("failed to set base attribute");
                        }
                    }
                }

                append_to_head(&el);

                Some(el.into())
            }

            Self::Link { attrs } => {
                let el = create_element::<HtmlLinkElement>("link");

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            add_class_list(&el, &*value);
                        }
                        "href" => {
                            el.set_href(value);
                        }
                        "rel" => {
                            el.set_rel(value);
                        }
                        _ => {
                            el.set_attribute(name, value)
                                .expect_throw("failed to set link attribute");
                        }
                    }
                }

                Some(el.into())
            }

            Self::Meta { attrs } => {
                let el = create_element::<HtmlMetaElement>("meta");

                for (name, value) in attrs.iter() {
                    match *name {
                        "class" => {
                            add_class_list(&el, &*value);
                        }
                        "name" => {
                            el.set_name(value);
                        }
                        "http-equiv" => {
                            el.set_http_equiv(value);
                        }
                        "content" => {
                            el.set_content(value);
                        }
                        "scheme" => {
                            el.set_scheme(value);
                        }
                        _ => {
                            el.set_attribute(name, value)
                                .expect_throw("failed to set link attribute");
                        }
                    }
                }

                Some(el.into())
            }
        }
    }

    pub fn detach(&self, element: Option<Element>) {
        if let Some(m) = element {
            m.parent_element().as_ref().map(|m| m.remove_child(m));
        }

        match self {
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
                                .expect_throw("failed to remove body attribute");
                        }
                    }
                }
            }

            Self::Title(_)
            | Self::Script { .. }
            | Self::Style { .. }
            | Self::Base { .. }
            | Self::Link { .. }
            | Self::Meta { .. } => {}
        }
    }
}
