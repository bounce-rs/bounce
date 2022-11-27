use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

use super::FormatTitle;
use gloo::utils::{body, document, document_element, head};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Element, HtmlBaseElement, HtmlElement, HtmlHeadElement, HtmlLinkElement, HtmlMetaElement,
    HtmlScriptElement, HtmlStyleElement,
};
use yew::prelude::*;

use crate::utils::Id;

thread_local! {
    static HEAD: HtmlHeadElement = head();
    static HTML_TAG: Element = document_element();
    static BODY_TAG: HtmlElement = body();
}

#[derive(PartialEq)]
pub(crate) struct HelmetState {
    pub tags: Vec<Arc<HelmetTag>>,
}

// TODO: fully type attributes for these elements.

/// An element supported by `<Helmet />` with its attributes and content.
///
/// You can use [`write_static`](Self::write_static) to write the content into a [`Write`](std::fmt::Write).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HelmetTag {
    /// `<title>...</title>`
    Title(Arc<str>),
    /// `<script ...>...</script>`
    Script {
        // we need to always render script as long as they are not the same tag, so we use an Id to
        // distinguish between them.
        #[doc(hidden)]
        _id: Id,
        /// The content of the tag.
        content: Arc<str>,
        /// The attributes of the tag.
        attrs: BTreeMap<Arc<str>, Arc<str>>,
    },
    /// `<style ...>...</style>`
    Style {
        /// The content of the tag.
        content: Arc<str>,
        /// The attributes of the tag.
        attrs: BTreeMap<Arc<str>, Arc<str>>,
    },
    /// `<html ...>`
    Html {
        /// The attributes of the tag.
        attrs: BTreeMap<Arc<str>, Arc<str>>,
    },
    /// `<body ...>`
    Body {
        /// The attributes of the tag.
        attrs: BTreeMap<Arc<str>, Arc<str>>,
    },
    /// `<base ... />`
    Base {
        /// The attributes of the tag.
        attrs: BTreeMap<Arc<str>, Arc<str>>,
    },
    /// `<link ... />`
    Link {
        /// The attributes of the tag.
        attrs: BTreeMap<Arc<str>, Arc<str>>,
    },
    /// `<meta ... />`
    Meta {
        /// The attributes of the tag.
        attrs: BTreeMap<Arc<str>, Arc<str>>,
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
    pub(crate) fn apply(&self) -> Option<Element> {
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
                    match name.as_ref() {
                        "type" => {
                            el.set_type(value);
                        }
                        "class" => {
                            add_class_list(&el, value);
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
                    match name.as_ref() {
                        "class" => {
                            add_class_list(&el, value);
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
                    match name.as_ref() {
                        "class" => {
                            add_class_list(&el, value);
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
                    match name.as_ref() {
                        "class" => {
                            add_class_list(&el, value);
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
                    match name.as_ref() {
                        "class" => {
                            add_class_list(&el, value);
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
                    match name.as_ref() {
                        "class" => {
                            add_class_list(&el, value);
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

                append_to_head(&el);

                Some(el.into())
            }

            Self::Meta { attrs } => {
                let el = create_element::<HtmlMetaElement>("meta");

                for (name, value) in attrs.iter() {
                    match name.as_ref() {
                        "class" => {
                            add_class_list(&el, value);
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
                                .expect_throw("failed to set meta attribute");
                        }
                    }
                }

                append_to_head(&el);

                Some(el.into())
            }
        }
    }

    pub(crate) fn detach(&self, element: Option<Element>) {
        if let Some(m) = element {
            m.parent_element()
                .as_ref()
                .map(|parent| parent.remove_child(&m));
        }

        match self {
            Self::Html { attrs } => {
                let el = HTML_TAG.with(|m| m.clone());

                for (name, value) in attrs.iter() {
                    match name.as_ref() {
                        "class" => {
                            remove_class_list(&el, value);
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
                    match name.as_ref() {
                        "class" => {
                            remove_class_list(&el, value);
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

/// Applies attributes on top of existing attributes.
fn merge_attrs(
    target: &mut BTreeMap<Arc<str>, Arc<str>>,
    current_attrs: &BTreeMap<Arc<str>, Arc<str>>,
) {
    for (name, value) in current_attrs.iter() {
        match name.as_ref() {
            "class" => match target.get("class").cloned() {
                Some(m) => {
                    target.insert(name.clone(), Arc::<str>::from(format!("{} {}", value, m)));
                }
                None => {
                    target.insert(name.clone(), value.clone());
                }
            },
            _ => {
                target.insert(name.clone(), value.clone());
            }
        }
    }
}

/// Merges helmet states into a set of tags to be rendered.
pub(super) fn merge_helmet_states(
    states: &[Rc<HelmetState>],
    format_title: Option<&FormatTitle>,
    default_title: Option<AttrValue>,
) -> BTreeSet<Arc<HelmetTag>> {
    let mut tags = BTreeSet::new();

    let mut title: Option<Arc<str>> = None;

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
                        (attrs.get("rel").cloned(), attrs.get("href").cloned()),
                        tag.clone(),
                    );
                }
                HelmetTag::Meta { ref attrs } => {
                    meta_tags.insert(
                        (
                            attrs.get("name").cloned(),
                            attrs.get("http-equiv").cloned(),
                            attrs.get("scheme").cloned(),
                            attrs.get("charset").cloned(),
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
            format_title
                .map(|fmt_fn| {
                    Arc::<str>::from(fmt_fn.emit(AttrValue::from(m.to_string())).to_string())
                })
                .unwrap_or(m)
        })
        .or_else(|| default_title.map(|m| m.to_string().into()))
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
